use std::path::PathBuf;

use crate::{
    plugin::{
        Plugin, PluginContent, PluginKind,
        host_api::{HostApi, HostApiVUnstable, HostCApiVUnstable, UnstableApi},
        manifests::Manifest,
    },
    prelude::{LunaticError, Result},
};

pub fn register_plugin(plugin:)

pub async fn load_plugin(path: PathBuf) -> Result<Plugin> {
    let manifest = match Manifest::load_manifest(path.clone()).await {
        Ok(o) => o,
        Err(e) => {
            return Err(LunaticError::InvalidManifest {
                reason: format!("{e}"),
            });
        }
    };
    Ok(Plugin {
        inner: match &manifest.get_kind() {
            PluginKind::Callbacks => {
                PluginContent::Callbacks(super::CallbackPlugin::load(&manifest).await?)
            }
            PluginKind::None => PluginContent::None,
            PluginKind::Mpsc => {
                todo!()
            }
        },
        ctx: super::PluginContext {
            host_api: match &manifest {
                Manifest::Unstable(m) => HostApi::Unstable(if m.rust_ver.is_some() {
                    UnstableApi::Rust(HostApiVUnstable::new())
                } else {
                    UnstableApi::C(HostCApiVUnstable::new())
                }),
            },
        },
        manifest,
        state: super::PluginState::Uninit,
    })
}
