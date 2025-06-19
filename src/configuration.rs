use std::env;
use config::{Config, Value};
use serde_aux::prelude::deserialize_number_from_string;
use serde::Deserialize;

/// Global settings.
#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    pub environment: String,
    pub application: ApplicationSettings
}

/// Application-specific settings.
/// 
/// Set default values in the `get_configuration` function.
#[derive(Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    /// Maximum number of in-flight requests before throttling.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub max_concurrent_requests: usize,
    /// Request timeout in seconds.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub request_timeout_s: u64,
}

/// Runtime environment
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub enum Environment {
    Local,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Prod => "prod",
        }
    }
}

// Note: `TryFrom` and `TryInto` are traits that allow for **fallible** type conversions.
//       Prefer `TryFrom` over `TryInto` if implementing just one of them.
//  Ref: https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html
impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "prod" => Ok(Environment::Prod),
            _ => Err(format!(
                "Unknown environment: {}. Use either `local` or `prod`.",
                value
            )),
        }
    }
}
// Note: `From` and `Into` are traits that allows for **infallible** type conversions.
//       Prefer `From` over `Into` if implementing just one of them.
//  Ref: https://doc.rust-lang.org/rust-by-example/conversion/from_into.html
impl From<Environment> for String {
    fn from(env: Environment) -> Self {
        env.as_str().into()
    }
}

impl From<Environment> for Value {
    fn from(env: Environment) -> Self {
        env.as_str().into()
    }
}

/// Reads and parses configurations from either YAML files or environment variables.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| Environment::Local.into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");  // Note: Safe to panic as it's not supposed to happen
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=8080 would set `Settings.application.port` to 8080.
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        // Setting default setting values.
        .set_default("application.host", "127.0.0.1")?
        .set_default("application.port", 8080)?
        .set_default("application.max_concurrent_requests", 10240)?
        .set_default("application.request_timeout_s", 20)?
        .build()?;

    settings.try_deserialize::<Settings>()
}
