use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub enum Environment {
    Local,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().trim() {
            "local" => Ok(Self::Local),
            "prod" => Ok(Self::Prod),
            unsupported_env => Err(format!(
                "{} is not a suppported environment. Use either 'local' or 'prod'",
                unsupported_env
            )),
        }
    }
}

/// Build up a layered configuration strategy where a base config
/// is first taken as the source and any other environment-specific
/// configuration file is merged with the base config, generating
/// the final config.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current dir.");
    let config_dir = base_path.join("config");

    // Define running environment. Default to local
    let environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENV");

    let env_name = format!("{}.yml", environment.as_str());

    tracing::info!("Reading config for env: {}", &env_name);

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.yml")))
        .add_source(config::File::from(config_dir.join(env_name)))
        .build()?;

    settings.try_deserialize::<Settings>()
}
