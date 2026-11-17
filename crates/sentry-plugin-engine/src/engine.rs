use crate::error::PluginEngineError;
use crate::event::PluginEvent;
use crate::run_mode::RunMode;
use acme_sentry_plugin_sdk_rs::context::RawContext;
use acme_sentry_plugin_sdk_rs::ffi::FfiStr;
use acme_sentry_plugin_sdk_rs::loader::{LoadedPlugin, UnloadPolicy, load_plugin};
use acme_sentry_plugin_sdk_rs::value::HostValue;
use acme_sentry_plugin_sdk_rs::vtable::HostVTable;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::JoinHandle;
use tracing::{Span, error, info, info_span};

pub const PLUGIN_DIRECTORY: &str = "plugins";

struct PluginRunState {
    #[allow(unused)]
    plugin_name: String,
    cancel_flag: Arc<AtomicBool>,
    events_tx: mpsc::Sender<PluginEvent>,
}

pub struct SetConfig {
    acme_token: String,
    domain: String,
    record_name: String,
}

impl SetConfig {
    pub fn new(acme_token: String, domain: String, record_name: String) -> Self {
        SetConfig {
            acme_token,
            domain,
            record_name,
        }
    }
}

#[allow(unused)]
extern "C" fn log_impl(state: *mut (), msg: FfiStr) {
    let state = unsafe { &*(state as *const PluginRunState) };
    let _ = state
        .events_tx
        .send(PluginEvent::Log(msg.as_str().to_owned()));
}

#[allow(unused)]
extern "C" fn should_cancel_impl(state: *mut ()) -> bool {
    let state = unsafe { &*(state as *const PluginRunState) };
    state.cancel_flag.load(Ordering::Relaxed)
}

#[allow(unused)]
static HOST_VTABLE: HostVTable = HostVTable {
    log: log_impl,
    should_cancel: should_cancel_impl,
};

pub struct PluginEngine;

impl PluginEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_unset_challenge(
        &self,
        default_plugin: PathBuf,
        run_config: SetConfig,
    ) -> Result<(), PluginEngineError> {
        self.load(default_plugin.as_path(), RunMode::Unset, run_config)
    }

    pub fn run_set_challenge(
        &self,
        default_plugin: PathBuf,
        run_config: SetConfig,
    ) -> Result<(), PluginEngineError> {
        self.load(default_plugin.as_path(), RunMode::Set, run_config)
    }

    fn load(
        &self,
        path: &Path,
        run_mode: RunMode,
        run_config: SetConfig,
    ) -> Result<(), PluginEngineError> {
        if path.extension().and_then(|ext| ext.to_str()) != Some("so") {
            return Err(PluginEngineError::PluginLoadError(
                "plugin is not of the required format".into(),
            ));
        }
        let plugin = match load_plugin(&path, UnloadPolicy::Unload) {
            Ok(p) => p,
            Err(err) => return Err(PluginEngineError::PluginLoadError(err)),
        };
        let plugin_name = plugin.name().to_string();
        info!("Running plugin: `{}`", plugin_name.clone());
        let config = HostValue::map(vec![
            ("domain", HostValue::str(run_config.domain.as_str())),
            ("acme_token", HostValue::str(run_config.acme_token.as_str())),
            (
                "record_name",
                HostValue::str(run_config.record_name.as_str()),
            ),
        ]);
        let (rx, _cancel_flag, worker) = self.run_in_sub_task(plugin, config, run_mode);
        let plugin_log_span = info_span!("plugin_log", plugin = plugin_name);
        plugin_log_span.follows_from(Span::current());
        let _ = plugin_log_span.enter();
        let mut error = None;
        for event in rx {
            match event {
                PluginEvent::Log(msg) => info!("{}", msg),
                PluginEvent::Finished(Ok(val)) => {
                    info!("{}", val.as_str().unwrap());
                }
                PluginEvent::Finished(Err(err)) => {
                    error!("{}", err);
                    error = Some(err.to_string());
                }
            }
        }
        let _ = worker.join();
        if let Some(err) = error {
            Err(PluginEngineError::PluginRuntimeError(err.to_string()))
        } else {
            Ok(())
        }
    }

    fn run_in_sub_task(
        &self,
        plugin: LoadedPlugin,
        config: HostValue,
        run_mode: RunMode,
    ) -> (Receiver<PluginEvent>, Arc<AtomicBool>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let worker_cancel_flag = cancel_flag.clone();

        let worker = thread::spawn(move || {
            let state = PluginRunState {
                plugin_name: plugin.name().to_owned(),
                cancel_flag: worker_cancel_flag,
                events_tx: tx.clone(),
            };
            let raw_context = RawContext {
                host_state: &state as *const PluginRunState as *mut (),
                vtable: &HOST_VTABLE,
            };
            let result = match plugin.validate_config(&raw_context, &config) {
                Err(e) => Err(e),
                Ok(()) => match run_mode {
                    RunMode::Set => plugin.set_acme_challenge(&raw_context, &config),
                    RunMode::Unset => plugin.unset_acme_challenge(&raw_context, &config),
                },
            };
            let _ = tx.send(PluginEvent::Finished(result));
        });
        (rx, cancel_flag, worker)
    }

    pub fn load_name_and_check_fingerprint(
        path: &Path,
    ) -> Result<(String, String), PluginEngineError> {
        let filename = path.file_name().clone().and_then(|n| n.to_str());
        let plugin = match load_plugin(&path, UnloadPolicy::Unload) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Error :: load_plugin returned error :: {}", err);
                return Ok((filename.unwrap_or("unknown").to_string(), err.to_string()));
            }
        };
        Ok((plugin.name().to_string().clone(), "Compatible".into()))
    }
}
