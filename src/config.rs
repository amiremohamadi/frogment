use log::LevelFilter;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_bind")]
    pub bind: String,

    #[serde(default, rename = "log-level")]
    pub log_level: LogLevel,

    #[serde(default)]
    pub dns: DnsConfig,
}

impl Config {
    pub fn new(config: &str) -> Result<Self, anyhow::Error> {
        Ok(serde_yaml::from_str(config)?)
    }
}

#[derive(Default, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub struct DnsConfig {
    pub upstream: String,
    pub level: usize,
    pub buffer: usize,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Info,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Info => LevelFilter::Info,
        }
    }
}

fn default_port() -> u16 {
    1999
}

fn default_bind() -> String {
    "127.0.0.1".to_string()
}
