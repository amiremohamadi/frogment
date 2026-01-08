pub mod config;
pub mod dns;

use std::sync::Arc;

use anyhow::Result;
use tokio::net::UdpSocket;

use crate::config::Config;
use crate::dns::{QName, rfc1035_fragment_qname};

pub struct Resolver {
    config: Config,
}

impl Resolver {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let addr = format!("{}:{}", self.config.bind, self.config.port);
        log::info!("bind to {}", addr);
        let socket = Arc::new(UdpSocket::bind(addr.clone()).await?);

        let mut buf = vec![0u8; self.config.dns.buffer];
        loop {
            let (size, client_addr) = socket.recv_from(&mut buf).await?;

            let resolver = self.clone();
            let socket = socket.clone();
            let packet = buf[..size].to_vec();

            tokio::spawn(async move {
                match resolver.resolve(&packet).await {
                    Ok(r) => {
                        let _ = socket.send_to(&r, &client_addr).await;
                    }
                    Err(e) => log::error!("error: {}", e),
                }
            });
        }
    }

    async fn resolve(self: Arc<Self>, packet: &Vec<u8>) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; self.config.dns.buffer];

        let qname = QName::from_bytes(&packet)?;
        log::info!("resolving {}", qname.name);

        let fragmented_packet = rfc1035_fragment_qname(packet, &qname, self.config.dns.level)?;

        let upstream = UdpSocket::bind("0.0.0.0:0").await?;
        upstream
            .send_to(&fragmented_packet, &self.config.dns.upstream)
            .await?;

        let (len, _) = upstream.recv_from(&mut buf).await?;
        Ok(buf[..len].to_vec())
    }
}
