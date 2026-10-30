use acme_sentry_plugin_sdk_rs::context::RawContext;
use acme_sentry_plugin_sdk_rs::ffi::FfiStr;
use acme_sentry_plugin_sdk_rs::loader::{LoadedPlugin, UnloadPolicy, load_plugin};
use acme_sentry_plugin_sdk_rs::value::HostValue;
use acme_sentry_plugin_sdk_rs::vtable::HostVTable;
use std::fs::read_dir;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::JoinHandle;
use tracing::info;
use crate::error::PluginEngineError;
use crate::event::PluginEvent;
use crate::run_mode::{DiscoveryType, RunMode};

struct PluginRunState {
    #[allow(unused)]
    plugin_name: String,
    cancel_flag: Arc<AtomicBool>,
    events_tx: mpsc::Sender<PluginEvent>,
}

#[allow(unused)]
extern "C" fn log_impl(state: *mut (), msg: FfiStr) {
    let state = unsafe { &*(state as *const PluginRunState) };
    let _ = state
        .events_tx
        .send(PluginEvent::Log(msg.as_str().to_owned()));
}

pub struct SetConfig {
    pub acme_token: String,
    pub domain: String,
    pub record_name: String,
}

#[allow(unused)]
static HOST_VTABLE: HostVTable = HostVTable {
    log: log_impl,
    should_cancel: should_cancel_impl,
};

#[allow(unused)]
extern "C" fn should_cancel_impl(state: *mut ()) -> bool {
    let state = unsafe { &*(state as *const PluginRunState) };
    state.cancel_flag.load(Ordering::Relaxed)
}

pub fn run_plugin(
    str_path: String,
    discovery_type: DiscoveryType,
    run_mode: RunMode,
    set_config: &SetConfig
) -> Result<(), PluginEngineError> {
    match discovery_type {
        DiscoveryType::Directory => {
            let Ok(entries) = read_dir(&str_path) else {
                return Err(PluginEngineError::PluginLoadError(
                    "Directory read not possible".to_string(),
                ));
            };

            for entry in entries.flatten() {
                load(&entry.path(), UnloadPolicy::Unload, run_mode.clone(), set_config)
            }
            Ok(())
        }
        DiscoveryType::SoloPlugin => {
            let path = Path::new(&str_path);
            load(&path, UnloadPolicy::Unload, run_mode, set_config);
            Ok(())
        }
    }
}

fn load(path: &Path, policy: UnloadPolicy, run_mode: RunMode, run_config: &SetConfig) {
    if path.extension().and_then(|ext| ext.to_str()) != Some("so") {
        return;
    }
    let plugin = match load_plugin(&path, policy) {
        Ok(p) => p,
        Err(_) => return,
    };
    info!("Running plugin: === {} ===", plugin.name().to_uppercase());
    let config = HostValue::map(vec![
        ("domain", HostValue::str(run_config.domain.as_str())),
        ("acme_token", HostValue::str(run_config.acme_token.as_str())),
        ("record_name", HostValue::str(run_config.record_name.as_str())),
    ]);
    let (rx, _cancel_flag, worker) = run_in_sub_task(plugin, config, run_mode);
    for event in rx {
        match event {
            PluginEvent::Log(msg) => info!("[PLUGIN_ENGINE]: {}", msg),
            PluginEvent::Finished(Ok(val)) => {
                info!("[PLUGIN_ENGINE]: {}", val.as_str().unwrap());
            }
            PluginEvent::Finished(Err(err)) => {
                info!("[PLUGIN_ENGINE]: {}", err);
            }
        }
    }
    let _ = worker.join();
}

fn run_in_sub_task(
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
