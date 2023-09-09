use clap::Parser;
// use dotenv;
use std::env;

/// Q&A web service API
#[derive(Parser, Debug, Default, serde::Deserialize, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    /// Which PORT the server is listening to
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "omid")]
    pub db_user: String,
    /// Database password
    #[clap(long, default_value = "password")]
    pub db_password: String,
    /// Url for postgres database
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// PORT number for database connection
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    /// Database naem
    #[clap(long, default_value = "rustwebdev")]
    pub db_name: String,
    // Web server port
    // port: u16,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        // dotenv::dotenv().ok();

        let config = Config::parse();

        if let Err(_) = env::var("BAD_WORDS_API_KEY") {
            panic!("BadWords API key not set.");
        }

        if let Err(_) = env::var("PASETO_KEY") {
            panic!("Paseto key not set.");
        }

        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(8080))
            .map_err(|e| handle_errors::Error::ParseError(e))?;

        let db_user = env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap();
        let db_host = env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("POSTGRES_PORT").unwrap_or(config.db_port.to_string());
        let db_name = env::var("POSTGRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(|e| handle_errors::Error::ParseError(e))?,
            db_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        env::set_var("BAD_WORDS_API_KEY", "yes");
        // env::set_var("PASETO_KEY", "yes");
        env::set_var("POSTGRES_USER", "user");
        env::set_var("POSTGRES_PASSWORD", "pass");
        env::set_var("POSTGRES_HOST", "localhost");
        env::set_var("POSTGRES_PORT", "5432");
        env::set_var("POSTGRES_DB", "rustwebdev");
    }

    // #[test]
    // fn unset_api_key() {
    //     let result = std::panic::catch_unwind(|| Config::new());
    //     assert!(result.is_err());
    // }

    #[test]
    fn unset_and_set_api_key() {
        // ENV VARIABLES ARE NOT SET
        let result = std::panic::catch_unwind(|| Config::new());
        assert!(result.is_err());

        // NOW SET SET ENV VARIABLES
        set_env();

        let expected = Config {
            log_level: "warn".to_string(),
            port: 8080,
            db_user: "user".to_string(),
            db_password: "pass".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "rustwebdev".to_string(),
        };

        let config = Config::new().unwrap();

        assert_eq!(config, expected);
    }
}
