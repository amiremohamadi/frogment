use anyhow::{Result, anyhow};

#[derive(Default, Debug)]
pub struct QName {
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub compressed: bool,
}

impl QName {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        // 0                   1                   2                   3
        // 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // |                      ID (16 bits)                         |  0-1
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // | QR |   Opcode  | AA | TC | RD | RA | Z | AD | CD | RCODE  |  2-3
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // |                  QDCOUNT (number of questions)            |  4-5
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // |                  ANCOUNT (number of answers)              |  6-7
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // |                  NSCOUNT (number of authority records)    |  8-9
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        // |                  ARCOUNT (number of additional records)   | 10-11
        // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

        if buf.len() < 12 {
            return Err(anyhow!("packet too small"));
        }

        // 12bytes header
        // qname starts right after the header
        let mut i = 12;
        let start = i;
        let mut labels = Vec::new();
        let mut jumped = false;

        loop {
            if i > buf.len() {
                return Err(anyhow!("out of bounds"));
            }

            let len = buf[i];

            // compression pointer
            if len & 0xc0 == 0xc0 {
                jumped = true;
                break;
            }

            // end of qname
            if len == 0 {
                i += 1;
                break;
            }

            if len > 63 {
                return Err(anyhow!("invalid label length"));
            }

            i += 1;
            if i + len as usize > buf.len() {
                return Err(anyhow!("out of bounds"));
            }

            let label = &buf[i..i + len as usize];
            labels
                .push(std::str::from_utf8(label).map_err(|_| anyhow!("unexpected char in label"))?);

            i += len as usize;
        }

        Ok(Self {
            start,
            end: i,
            name: labels.join("."),
            compressed: jumped,
        })
    }
}

pub fn rfc1035_fragment_qname(buf: &Vec<u8>, qname: &QName, jumps: usize) -> Result<Vec<u8>> {
    if qname.compressed {
        return Err(anyhow!("qname already compressed"));
    }

    let packet = &mut buf[..qname.start].to_vec();

    // dummy pointer based on rfc1035
    packet.push(0xC0);
    packet.push(0x00);

    // qtype, qclass, opt, ...
    packet.extend_from_slice(&buf[qname.end..]);

    // rest of dummy pointers
    let ptrs_offset = packet.len();
    for _ in 0..jumps {
        packet.push(0xC0);
        packet.push(0x00);
    }

    let qname_offset = packet.len();
    packet.extend_from_slice(&buf[qname.start..qname.end]);

    // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4
    //
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |1 1|            OFFSET                         |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    //
    // now we need to patch the pointers to point to the real qname
    let ptr = 0xC000 | (qname_offset as u16);

    packet[qname.start] = (ptr >> 8) as u8;
    packet[qname.start + 1] = ptr as u8;
    for i in 0..jumps {
        let pos = ptrs_offset + i * 2;
        packet[pos] = (ptr >> 8) as u8;
        packet[pos + 1] = ptr as u8;
    }

    Ok(packet.to_vec())
}
