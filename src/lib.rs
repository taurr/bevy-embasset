#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]

mod plugin;
pub use plugin::EmbassetPlugin;

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::*;

use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture},
    prelude::*,
};
use derive_more::DebugCustom;
use smol_str::SmolStr;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Trait for easy replacement of the default [`AssetServer`](bevy::asset::AssetServer).
///
/// # Example
/// ```rust
/// # use bevy::{prelude::*, asset::AssetPlugin};
/// # use bevy_embasset::AddEmbassetPlugin;
/// # fn main() {
///     App::new().add_embasset_plugin(add_embasset_assets);
///
///     // Add assets to Embasset manually
///     // fn void add_embasset_assets(io: &mut EmbassetIo) {
///     //   ...
///     // }
///
///     // Or, use the buildscript, and just include the function
///     // include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));
/// # }
/// # fn add_embasset_assets(#[allow(unused)] in_memory: &mut bevy_embasset::EmbassetIo){
/// # }
/// ```
pub trait AddEmbassetPlugin {
    /// Replace the default [`AssetServer`](bevy::asset::AssetServer).
    fn add_embasset_plugin<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: Fn(&mut EmbassetIo) + Send + Sync + 'static;
}

impl AddEmbassetPlugin for App {
    fn add_embasset_plugin<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: Fn(&mut EmbassetIo) + Send + Sync + 'static,
    {
        self.add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(EmbassetPlugin::new(config_fn))
        })
    }
}

/// Defines another [`AssetServer`](bevy::asset::AssetServer) that may be used for loading assets
/// by prepending the asset path with a custom string.
#[derive(DebugCustom)]
#[debug(fmt = "AssetIoAlternative {{ path_start = {} }}", path_start)]
pub struct AssetIoAlternative {
    path_start: SmolStr,
    fallback_on_err: bool,
    asset_io: Box<dyn AssetIo>,
}

impl AssetIoAlternative {
    /// Creates a new `AssetIoAlternative`.
    ///
    /// - **path_start**
    ///
    ///     Any asset whose path is prepended with this will be handed of to the specified [`AssetIo`](bevy::asset::AssetIo).
    ///
    /// - **asset_io**
    ///
    ///     [`AssetServer`](bevy::asset::AssetServer) for loading assets.
    pub fn new<T: AssetIo>(path_start: &str, asset_io: T) -> Self {
        AssetIoAlternative {
            path_start: SmolStr::new(path_start),
            fallback_on_err: false,
            asset_io: Box::new(asset_io),
        }
    }

    /// If loading an asset with this alternate [`AssetIo`](bevy::asset::AssetIo) fails, fallback
    /// to the default - either file or embedded resources, but withouth the `path_start` part of the path.
    ///
    /// If `bevy_embasset` is compiled with the `use-default-assetio` feature, fallback will attempt
    /// to find a file first and if that fails too, try to use an embedded resource.
    pub fn fallback_on_err(mut self) -> Self {
        self.fallback_on_err = true;
        self
    }
}

/// Custom [`AssetServer`](bevy::asset::AssetServer), that can load assets embedded into the binary,
/// or use other servers for handling the load.
#[derive(DebugCustom)]
#[debug(fmt = "EmbassetIo {{ handlers={:?} }}", handlers)]
pub struct EmbassetIo {
    #[cfg(feature = "use-default-assetio")]
    default_io: Box<dyn AssetIo>,
    handlers: Vec<AssetIoAlternative>,
    embedded_resources: HashMap<&'static Path, &'static [u8]>,
}

#[cfg(not(feature = "use-default-assetio"))]
impl Default for EmbassetIo {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbassetIo {
    /// Create a new instance of the custom [`AssetServer`](bevy::asset::AssetServer).
    ///
    /// # Requires
    ///
    /// Feature: `use-default-assetio`
    #[cfg(feature = "use-default-assetio")]
    pub(crate) fn with_default_assetio(default_io: Box<dyn AssetIo>) -> Self {
        EmbassetIo {
            default_io,
            handlers: Default::default(),
            embedded_resources: Default::default(),
        }
    }

    /// Create a new instance of the custom [`AssetServer`](bevy::asset::AssetServer).
    #[cfg(not(feature = "use-default-assetio"))]
    pub(crate) fn new() -> Self {
        EmbassetIo {
            handlers: Default::default(),
            embedded_resources: Default::default(),
        }
    }

    /// Add a custom [`AssetServer`](bevy::asset::AssetServer) for handling specific paths.
    pub fn add_handler(&mut self, handler: AssetIoAlternative) -> &mut Self {
        self.handlers.push(handler);
        self
    }

    /// Add a slice of bytes as a resource using the specified Path.
    pub fn add_embedded_asset(&mut self, path: &'static Path, data: &'static [u8]) -> &mut Self {
        self.embedded_resources.insert(path, data);
        self
    }

    /// Get the data from the asset matching the path provided.
    ///
    /// # Errors
    ///
    /// This will returns an error if the path is not known or embedded.
    #[doc(hidden)]
    pub fn load_embedded_path_sync(&self, path: &Path) -> Result<Vec<u8>, AssetIoError> {
        self.embedded_resources
            .get(path)
            .map(|b| b.to_vec())
            .ok_or_else(|| bevy::asset::AssetIoError::NotFound(path.to_path_buf()))
    }
}

async fn load_path_via_assetio<'a>(
    path: &'a Path,
    config: &'a AssetIoAlternative,
    bevasset: &'a EmbassetIo,
) -> Result<Vec<u8>, AssetIoError> {
    // first remove the path_start part of the path
    let path = path.display().to_string();
    let path = path
        .strip_prefix(config.path_start.as_str())
        .expect("path does not start with the defined path_start");
    let path = Path::new(path);

    // now load using the handler
    trace!(?path, path_start=?config.path_start, "load asset via AssetIo");
    let r = config.asset_io.load_path(Path::new(path)).await;

    // fallback in case of errors
    match r {
        r @ Ok(_) => {
            trace!(?path, "loaded");
            r
        }
        Err(err) if config.fallback_on_err => {
            info!(?err, ?path, path_start=?config.path_start, "failed loading asset using handler, fallback to default");
            bevasset.load_path(path).await
        }
        Err(err) => {
            warn!(?err, ?path, path_start=?config.path_start, "failed loading asset");
            Err(err)
        }
    }
}

#[cfg(feature = "use-default-assetio")]
async fn load_path<'a>(path: &'a Path, bevasset: &'a EmbassetIo) -> Result<Vec<u8>, AssetIoError> {
    if let Some(config) = bevasset
        .handlers
        .iter()
        .find(|h| path.starts_with(h.path_start.as_str()))
    {
        load_path_via_assetio(path, config, bevasset).await
    } else {
        trace!(?path, "load asset via default AssetIo");
        match bevasset.default_io.load_path(path).await {
            r @ Ok(_) => {
                trace!(?path, "loaded");
                r
            }
            Err(err) => {
                info!(
                    ?err,
                    ?path,
                    "failed loading asset using default AssetIo, fallback to embedded resource"
                );
                match bevasset
                    .embedded_resources
                    .get(path)
                    .map(|b| b.to_vec())
                    .ok_or_else(|| bevy::asset::AssetIoError::NotFound(path.to_path_buf()))
                {
                    r @ Ok(_) => {
                        trace!(?path, "loaded");
                        r
                    }
                    Err(err) => {
                        warn!(?err, ?path, "failed loading asset");
                        Err(err)
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "use-default-assetio"))]
async fn load_path<'a>(path: &'a Path, bevasset: &'a EmbassetIo) -> Result<Vec<u8>, AssetIoError> {
    if let Some(config) = bevasset
        .handlers
        .iter()
        .find(|h| path.starts_with(h.path_start.as_str()))
    {
        load_path_via_assetio(path, config, bevasset).await
    } else {
        trace!(?path, "load asset as embedded resource");
        match bevasset
            .embedded_resources
            .get(path)
            .map(|b| b.to_vec())
            .ok_or_else(|| bevy::asset::AssetIoError::NotFound(path.to_path_buf()))
        {
            r @ Ok(_) => {
                trace!(?path, "loaded");
                r
            }
            Err(err) => {
                warn!(?err, ?path, "failed loading asset");
                Err(err)
            }
        }
    }
}

fn read_embedded_directory(
    bevasset: &EmbassetIo,
    path: &Path,
) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
    trace!(?path, "read directory as embedded resource");
    if bevasset.is_directory(path) {
        #[allow(clippy::needless_collect)]
        let paths: Vec<_> = bevasset
            .embedded_resources
            .keys()
            .filter(|loaded_path| loaded_path.starts_with(path))
            .map(|t| t.to_path_buf())
            .collect();
        trace!(?path, "loaded");
        Ok(Box::new(paths.into_iter()))
    } else {
        let err = AssetIoError::Io(std::io::ErrorKind::NotFound.into());
        warn!(?err, ?path, "failed read directory");
        Err(err)
    }
}

#[cfg(feature = "use-default-assetio")]
impl AssetIo for EmbassetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(load_path(path, self))
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            // first remove the path_start part of the path
            let path = path.display().to_string();
            let path = path
                .strip_prefix(config.path_start.as_str())
                .expect("path does not start with the defined path_start");
            let path = Path::new(path);
            // pass call to handler
            trace!(?path, path_start=?config.path_start, "read directory via handler");
            config.asset_io.read_directory(path)
        } else {
            trace!(?path, "read directory via default AssetIo");
            match self.default_io.read_directory(path) {
                r @ Ok(_) => r,
                Err(err) => {
                    info!(
                        ?err,
                        ?path,
                        "failed read directory via default AssetIo, fallback to embedded resource"
                    );
                    read_embedded_directory(self, path)
                }
            }
        }
    }

    fn is_directory(&self, path: &Path) -> bool {
        let is_directory = if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            config.asset_io.is_directory(path)
        } else {
            // here there's no chance of doing a fallback.
            // if default_io is enabled, it effectively dictates the result when not using a
            // matched path_start
            self.default_io.is_directory(path)
        };
        is_directory
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            config.asset_io.watch_path_for_changes(path)
        } else {
            match self.default_io.watch_path_for_changes(path) {
                r @ Ok(_) => r,
                Err(_) => Ok(()),
            }
        }
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        match self.default_io.watch_for_changes() {
            r @ Ok(_) => r,
            Err(_) => Ok(()),
        }
    }
}

#[cfg(not(feature = "use-default-assetio"))]
impl AssetIo for EmbassetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(load_path(path, self))
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            // first remove the path_start part of the path
            let path = path.display().to_string();
            let path = path
                .strip_prefix(config.path_start.as_str())
                .expect("path does not start with the defined path_start");
            let path = Path::new(path);
            // pass call to handler
            trace!(?path, path_start=?config.path_start, "read directory via handler");
            config.asset_io.read_directory(path)
        } else {
            read_embedded_directory(self, path)
        }
    }

    fn is_directory(&self, path: &Path) -> bool {
        let is_directory = if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            config.asset_io.is_directory(path)
        } else {
            let as_folder = path.join("");
            self.embedded_resources
                .keys()
                .any(|loaded_path| loaded_path.starts_with(&as_folder) && loaded_path != &path)
        };
        is_directory
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.path_start.as_str()))
        {
            config.asset_io.watch_path_for_changes(path)
        } else {
            Ok(())
        }
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(feature = "use-default-assetio"))]
mod tests {
    use bevy::asset::AssetIo;
    use std::path::Path;

    use super::EmbassetIo;

    #[test]
    fn load_path() {
        let mut embedded = EmbassetIo::new();
        embedded.add_embedded_asset(Path::new("asset.png"), &[1, 2, 3]);
        embedded.add_embedded_asset(Path::new("other_asset.png"), &[4, 5, 6]);

        assert!(embedded
            .load_embedded_path_sync(Path::new("asset.png"))
            .is_ok());
        assert_eq!(
            embedded
                .load_embedded_path_sync(Path::new("asset.png"))
                .unwrap(),
            [1, 2, 3]
        );
        assert_eq!(
            embedded
                .load_embedded_path_sync(Path::new("other_asset.png"))
                .unwrap(),
            [4, 5, 6]
        );
        assert!(embedded
            .load_embedded_path_sync(Path::new("asset"))
            .is_err());
        assert!(embedded
            .load_embedded_path_sync(Path::new("other"))
            .is_err());
    }

    #[test]
    fn is_directory() {
        let mut embedded = EmbassetIo::new();
        embedded.add_embedded_asset(Path::new("asset.png"), &[]);
        embedded.add_embedded_asset(Path::new("directory/asset.png"), &[]);

        assert!(!embedded.is_directory(Path::new("asset.png")));
        assert!(!embedded.is_directory(Path::new("asset")));
        assert!(embedded.is_directory(Path::new("directory")));
        assert!(embedded.is_directory(Path::new("directory/")));
        assert!(!embedded.is_directory(Path::new("directory/asset")));
    }

    #[test]
    fn read_directory() {
        let mut embedded = EmbassetIo::new();
        embedded.add_embedded_asset(Path::new("asset.png"), &[]);
        embedded.add_embedded_asset(Path::new("directory/asset.png"), &[]);
        embedded.add_embedded_asset(Path::new("directory/asset2.png"), &[]);

        assert!(embedded.read_directory(Path::new("asset.png")).is_err());
        assert!(embedded.read_directory(Path::new("directory")).is_ok());
        let mut list = embedded
            .read_directory(Path::new("directory"))
            .unwrap()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        list.sort();
        assert_eq!(list, vec!["directory/asset.png", "directory/asset2.png"]);
    }
}
