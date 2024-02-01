use config::{builder::DefaultState, ConfigBuilder, Map, Value};
use log::Level;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::{
    deserialize_number_from_string, deserialize_option_number_from_string,
};
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};
pub const APP_NAME: &str = env!("APP_NAME");

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize, Serialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub loglevel: Level,
}

#[derive(Deserialize, Serialize)]
pub struct DatabaseSettings {
    pub driver: String,
    pub host: Option<String>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    #[serde(default)]
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub require_ssl: bool,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum Environment {
    Local,
    Test,
    Production,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
            loglevel: Level::Info,
        }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            driver: "sqlite".to_string(),
            host: None,
            port: None,
            username: None,
            password: None,
            name: None,
            require_ssl: false,
            path: Some(format!("{}.db", APP_NAME)),
        }
    }
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Test => "test",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "test" => Ok(Self::Test),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `test` or `production`.",
                other
            )),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Settings, config::ConfigError> {
        let configuration_directory = std::env::var("CONFIG_DIR").map_or(
            dirs::config_dir()
                .expect("Configuration directory could not found")
                .join(Path::new(APP_NAME)),
            PathBuf::from,
        );
        let data_directory = std::env::var("DATA_DIR").map_or(
            dirs::data_dir()
                .expect("Data directory could not found")
                .join(Path::new(APP_NAME)),
            PathBuf::from,
        );

        if !configuration_directory.exists() {
            std::fs::create_dir_all(configuration_directory.clone())
                .expect("Cannot create configuration directory");
        }

        if !data_directory.exists() {
            std::fs::create_dir_all(data_directory.clone()).expect("Cannot create data directory");
        }

        let runtime_environment: Environment = std::env::var("APP_ENVIRONMENT")
            .map_or(Environment::Local, |s| {
                s.try_into().expect("Failed to parse APP_ENVIRONMENT")
            });

        let mut environment_filename: String = "".to_string();
        if configuration_directory
            .join(format!("{}.yaml", runtime_environment.as_str()))
            .exists()
        {
            environment_filename = format!("{}.yaml", runtime_environment.as_str());
        } else if configuration_directory
            .join(format!("{}.yml", runtime_environment.as_str()))
            .exists()
        {
            environment_filename = format!("{}.yml", runtime_environment.as_str());
        }

        let mut config_builder: ConfigBuilder<DefaultState> = config::Config::builder()
            .set_default(
                "application",
                serde_yaml::from_value::<Map<String, Value>>(
                    serde_yaml::to_value(ApplicationSettings::default()).unwrap(),
                )
                .unwrap(),
            )?
            .set_default(
                "database",
                serde_yaml::from_value::<Map<String, Value>>(
                    serde_yaml::to_value(DatabaseSettings::default()).unwrap(),
                )
                .unwrap(),
            )?;
        if !environment_filename.is_empty() {
            let configuration_file: PathBuf =
                configuration_directory.join(environment_filename.clone());
            if configuration_file.exists() {
                config_builder = config_builder.add_source(config::File::from(configuration_file));
            }
        }
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        config_builder = config_builder.add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );
        let config = config_builder.build()?;
        let mut settings = config.try_deserialize::<Settings>()?;

        if settings
            .database
            .path
            .clone()
            .is_some_and(|path| !Path::new(&path).is_absolute())
        {
            settings.database.path = Some(
                data_directory
                    .join(Path::new(&settings.database.path.unwrap()))
                    .display()
                    .to_string(),
            );
        }

        Self::check(settings)
    }

    fn check(settings: Settings) -> Result<Settings, config::ConfigError> {
        if settings
            .database
            .driver
            .clone()
            .to_lowercase()
            .eq(&String::from("sqlite").to_lowercase())
            && settings
                .database
                .path
                .clone()
                .is_some_and(|value| !Path::new(&value).exists())
        {
            println!("Creating database file");
            std::fs::File::create(Path::new(&settings.database.path.clone().unwrap()))
                .expect("Cannot create sqlite database file");
        }
        Ok(settings)
    }
}

impl DatabaseSettings {
    pub fn get_url(&self) -> String {
        if self.driver == "sqlite" {
            return format!("{}://{}", self.driver.clone(), self.path.clone().unwrap()).to_owned();
        }
        if !self.username.clone().unwrap_or_default().is_empty() {
            return format!(
                "{}://{}:{}@{}:{}/{}",
                self.driver.clone(),
                self.username.clone().unwrap(),
                self.password.clone().unwrap_or_default(),
                self.host.clone().unwrap(),
                self.port.unwrap(),
                self.name.clone().unwrap()
            )
            .to_owned();
        }
        format!(
            "{}://{}:{}/{}",
            self.driver.clone(),
            self.host.clone().unwrap(),
            self.port.unwrap(),
            self.name.clone().unwrap()
        )
        .to_owned()
    }
}
