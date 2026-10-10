use acme_sentry_plugin_sdk_rs::error::HostError;
use acme_sentry_plugin_sdk_rs::value::HostValue;

pub enum PluginEvent {
    Log(String),
    Finished(Result<HostValue, HostError>)
}