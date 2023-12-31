use config::{Config as ConfigRs, ConfigError, Environment, File};
use log::debug;
use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "/etc/niftymic/niftymic.toml";
const DEFAULT_CONFIG: &str = "config/default.toml";
const ENV_PREFIX: &str = "NIFTYMIC";

#[derive(Debug, Deserialize, Clone)]
pub struct Docker {
    pub image: String,
    pub working_directory: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Executable {
    pub dcm2niix: String,
    pub docker: String,
    pub medcon: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Output {
    pub base_directory: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Telegram {
    pub enable: bool,
    pub teloxide_token: String,
    pub channel_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub output: Output,
    pub executables: Executable,
    pub docker: Docker,
    pub telegram: Option<Telegram>,
}

impl Config {
    pub fn new(path: Option<String>) -> Result<Config, ConfigError> {
        let config = match path {
            Some(config) => config,
            None => DEFAULT_CONFIG_PATH.to_string(),
        };
        debug!("Reading configuration");
        let result = ConfigRs::builder()
            .add_source(File::with_name(DEFAULT_CONFIG))
            .add_source(File::with_name(&config).required(false))
            .add_source(
                Environment::with_prefix(ENV_PREFIX)
                    .prefix_separator("_")
                    .separator("_")
                    .ignore_empty(true)
                    .list_separator(" "),
            )
            .build()?;
        debug!("Try deserializing configuration");
        result.try_deserialize()
    }
}
