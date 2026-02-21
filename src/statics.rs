use std::option::Option;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    version: bool,
    #[arg(long, default_value_t = false, help = "Enable verbose logging")]
    verbose: bool,
    #[arg(short, long, default_value_t = false, help = "Enable job-self-shutdown (buggy, will cause application to stall if job fails)")]
    wait_for_shutdown: bool,
    #[arg(short, long, default_value = "ec-p256", help = "Specify what key type, that acme-sentry should use to log in to the CA with")]
    requested_login_key_type: String,
    #[arg(long, help = "ACME system base url")]
    acme_base_url: String,
    #[arg(short, long, default_value = "/opt/acme-sentry", help = "Output directory of generated files")]
    dir_out: String,
}

impl Args {
    pub fn acme_base_url(&self) -> &str {
        &self.acme_base_url
    }
    pub fn version(&self) -> bool {
        self.version
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }
    pub fn dir_out(&self) -> &str {
        &self.dir_out
    }
    pub fn wait_for_shutdown(&self) -> bool {
        self.wait_for_shutdown
    }
    pub fn requested_login_key_type(&self) -> &str {
        &self.requested_login_key_type
    }
}