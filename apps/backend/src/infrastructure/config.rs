use std::env;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    pub fn is_development(self) -> bool {
        matches!(self, AppEnv::Development)
    }
}

impl FromStr for AppEnv {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(AppEnv::Development),
            "production" | "prod" => Ok(AppEnv::Production),
            other => Err(ConfigError::InvalidAppEnv(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: AppEnv,
    pub database_url: String,
    pub port: u16,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required environment variable `{0}`")]
    Missing(&'static str),
    #[error("invalid APP_ENV `{0}` (expected `development` or `production`)")]
    InvalidAppEnv(String),
    #[error("invalid PORT `{0}` (expected a number in 0..=65535)")]
    InvalidPort(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_source(|key| env::var(key).ok())
    }

    /// Parses config from an arbitrary key lookup so it stays unit-testable.
    pub fn from_source(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let app_env = match get("APP_ENV") {
            Some(raw) if !raw.trim().is_empty() => raw.parse()?,
            _ => AppEnv::Development,
        };

        let database_url = get("DATABASE_URL")
            .filter(|v| !v.trim().is_empty())
            .ok_or(ConfigError::Missing("DATABASE_URL"))?;

        let port = match get("PORT") {
            Some(raw) if !raw.trim().is_empty() => raw
                .trim()
                .parse::<u16>()
                .map_err(|_| ConfigError::InvalidPort(raw))?,
            _ => 5000,
        };

        Ok(Config {
            app_env,
            database_url,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn source(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    #[test]
    fn defaults_apply_when_only_database_url_is_set() {
        let cfg = Config::from_source(source(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(cfg.app_env, AppEnv::Development);
        assert_eq!(cfg.port, 5000);
        assert_eq!(cfg.database_url, "postgres://x");
    }

    #[test]
    fn parses_production_env_and_custom_port() {
        let cfg = Config::from_source(source(&[
            ("APP_ENV", "production"),
            ("DATABASE_URL", "postgres://x"),
            ("PORT", "8080"),
        ]))
        .unwrap();
        assert_eq!(cfg.app_env, AppEnv::Production);
        assert!(!cfg.app_env.is_development());
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn app_env_is_case_insensitive_and_accepts_short_forms() {
        let cfg = Config::from_source(source(&[("APP_ENV", "  Prod "), ("DATABASE_URL", "x")]))
            .unwrap();
        assert_eq!(cfg.app_env, AppEnv::Production);
    }

    #[test]
    fn empty_app_env_falls_back_to_development() {
        let cfg = Config::from_source(source(&[("APP_ENV", "   "), ("DATABASE_URL", "x")])).unwrap();
        assert_eq!(cfg.app_env, AppEnv::Development);
    }

    #[test]
    fn invalid_app_env_is_rejected() {
        let err = Config::from_source(source(&[("APP_ENV", "staging"), ("DATABASE_URL", "x")]))
            .unwrap_err();
        assert_eq!(err, ConfigError::InvalidAppEnv("staging".to_owned()));
    }

    #[test]
    fn missing_database_url_is_rejected() {
        let err = Config::from_source(source(&[("APP_ENV", "development")])).unwrap_err();
        assert_eq!(err, ConfigError::Missing("DATABASE_URL"));
    }

    #[test]
    fn blank_database_url_is_rejected() {
        let err = Config::from_source(source(&[("DATABASE_URL", "   ")])).unwrap_err();
        assert_eq!(err, ConfigError::Missing("DATABASE_URL"));
    }

    #[test]
    fn invalid_port_is_rejected() {
        let err =
            Config::from_source(source(&[("DATABASE_URL", "x"), ("PORT", "not-a-port")]))
                .unwrap_err();
        assert_eq!(err, ConfigError::InvalidPort("not-a-port".to_owned()));
    }

    #[test]
    fn out_of_range_port_is_rejected() {
        let err =
            Config::from_source(source(&[("DATABASE_URL", "x"), ("PORT", "70000")])).unwrap_err();
        assert_eq!(err, ConfigError::InvalidPort("70000".to_owned()));
    }
}
