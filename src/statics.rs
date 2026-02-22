use clap::Parser;
use std::option::Option;
use tracing::Level;

#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,
    #[arg(short, long, default_value = "info", value_parser = log_level_parse, help = "Enable verbose logging")]
    pub logging_level: Level,
    #[arg(long, default_value_t = false, help = "Enable application-mode (input is required from user to terminate application)")]
    pub application_mode: bool,
    #[arg(long, default_value = "ec-p256", help = "Specify what key type, that acme-sentry should use to log in to the CA with")]
    pub requested_login_key_type: String,
    #[arg(long, help = "ACME system base url")]
    pub acme_base_url: Option<String>,
    #[arg(long, default_value = "/opt/acme-sentry", help = "Output directory of generated files")]
    pub dir_out: String,
    #[arg(long, help = "User id that acme-sentry shall persist reference to if this is not present, acme-sentry will generate a new user")]
    pub with_user_id: Option<String>,
    #[arg(long, help = "Email that acme-sentry shall try to connect with the user")]
    pub with_email: Option<String>,
}

fn log_level_parse(s: &str) -> Result<Level, String> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Ok(Level::INFO),
    }
}