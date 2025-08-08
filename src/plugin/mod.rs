use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use crossbeam::channel::Sender;
use libloading::Library;
use lunatic_macros::load_symbol_c;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    mailbox::Endpoint,
    plugin::{
        host_api::HostApi,
        manifests::{Manifest, ManifestVersion},
    },
    prelude::*,
};
pub mod host_api;
pub mod manifests;
pub mod registrar;

unsafe impl Send for Plugin {}

/// Uniform Plugin struct that gets registered to every bus.
pub struct Plugin {
    manifest: Manifest,
    ctx: PluginContext,
    state: PluginState,
    inner: PluginContent,
}

pub enum ConfigData {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec(Vec<ConfigData>),
}

pub struct PluginContext {
    host_api: HostApi,
}

pub enum PluginContent {
    None,
    Callbacks(CallbackPlugin),
    Sender(Sender<Envelope>),
}

unsafe fn to_raw_plugin<T>(value: T) -> *mut u8 {
    Box::into_raw(Box::new(value)) as *mut u8
}

unsafe fn free_raw<T>(ptr: *mut u8) {
    unsafe { drop(Box::from_raw(ptr as *mut T)) }
}

impl Endpoint for Plugin {
    fn receive(&self, envelope: Envelope) -> NResult {
        match &self.inner {
            PluginContent::None => Ok(()),
            PluginContent::Sender(sender) => Ok(sender.send(envelope)?),
            PluginContent::Callbacks(plugin) => plugin.send(envelope),
        }
    }
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum PluginState {
    Ready = 0,
    Uninit = 1,
    Busy = 2,
    Unresponsive = 3,
    Killed = 4,
    Dead = 5,
    Error = 6,
}

pub struct CallbackPlugin {
    lib: Library,
    inner: CallbackPluginVersion,
}

impl CallbackPlugin {
    pub fn send(&self, envelope: Envelope) -> NResult {
        unsafe {
            match &self.inner {
                CallbackPluginVersion::Unstable(callback) => (match callback.receive {
                    Some(f) => f,
                    None => reject_envelope,
                })(envelope),
            }
        }
    }
}

impl CallbackPlugin {
    pub async fn load(manifest: &Manifest) -> Result<Self> {
        let lib = unsafe { Library::new(manifest.get_src()) }.map_err(|e| {
            LunaticError::PluginLoadFailed {
                path: manifest.get_src().to_path_buf(),
                reason: format!("Failed to load the plugin library: {e}"),
            }
        })?;
        if manifest.get_version() == ManifestVersion::Unstable {
            Ok(CallbackPlugin {
                inner: CallbackPluginVersion::Unstable(CallbackPluginUnstable {
                    init: load_symbol_c!(&lib, "init", ()->NResult),
                    receive: load_symbol_c!(&lib, "receive", (Envelope)->NResult),
                    import: load_symbol_c!(&lib, "import", (Value)->NResult),
                    export: load_symbol_c!(&lib, "export", ()->Value),
                }),
                lib,
            })
        } else {
            unreachable!()
        }
    }
}

unsafe extern "C" fn reject_envelope(envelope: Envelope) -> NResult {
    Err(LunaticError::PluginFailedMessage { envelope })
}

pub enum CallbackPluginVersion {
    Unstable(CallbackPluginUnstable),
}

pub struct CallbackPluginUnstable {
    /// Optional init. Introduces the Host API. Will be cleared after use.
    init: Option<unsafe extern "C" fn() -> NResult>,
    /// Handle incoming envelope
    receive: Option<unsafe extern "C" fn(Envelope) -> NResult>,
    /// Configuration from external json.
    import: Option<unsafe extern "C" fn(serde_json::Value) -> NResult>,
    /// Save call
    export: Option<unsafe extern "C" fn() -> serde_json::Value>,
}

#[derive(Deserialize)]
pub enum PluginKind {
    None,
    Callbacks,
    Mpsc,
}
