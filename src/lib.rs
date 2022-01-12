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
pub use plugin::BevassetIoPlugin;

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::generate_include_all_assets;

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

/// Defines another [`AssetServer`](bevy::asset::AssetServer) that may be used for loading assets
/// by prepending their path with a custom string.
#[derive(DebugCustom)]
#[debug(fmt = "HandlerConfig {{ protocol = {} }}", protocol)]
pub struct HandlerConfig {
    protocol: SmolStr,
    fallback_on_err: bool,
    asset_io: Box<dyn AssetIo>,
}

impl HandlerConfig {
    /// Creates a new `HandlerConfig`.
    ///
    /// - **protocol**
    ///
    ///     The actual [`AssetIo`](bevy::asset::AssetIo) that handles a request is identified
    ///     through the first characters of its path.
    ///
    /// - **asset_io**
    ///
    ///     Asset Server for loading any asset using a path starting with `protocol`.
    pub fn new<T: AssetIo>(protocol: &str, asset_io: T) -> Self {
        HandlerConfig {
            protocol: SmolStr::new(protocol),
            fallback_on_err: false,
            asset_io: Box::new(asset_io),
        }
    }

    /// If loading an asset through this handler fails, fallback to the default - either file or
    /// embedded resources, but withouth the `protocol` part of the path.
    pub fn fallback_on_err(mut self) -> Self {
        self.fallback_on_err = true;
        self
    }
}

/// Custom [`AssetServer`](bevy::asset::AssetServer), that can load assets embedded into the binary,
/// or use other servers for handling the load.
#[derive(DebugCustom)]
#[debug(fmt = "BevassetIo {{ handlers={:?} }}", handlers)]
pub struct BevassetIo {
    #[cfg(feature = "use-default-assetio")]
    default_io: Box<dyn AssetIo>,

    handlers: Vec<HandlerConfig>,
    embedded_resources: HashMap<&'static Path, &'static [u8]>,
}

#[cfg(not(feature = "use-default-assetio"))]
impl Default for BevassetIo {
    fn default() -> Self {
        Self::new()
    }
}

impl BevassetIo {
    /// Create a new instance of the custom [`AssetServer`](bevy::asset::AssetServer).
    #[cfg(feature = "use-default-assetio")]
    pub fn new(default_io: Box<dyn AssetIo>) -> Self {
        BevassetIo {
            default_io,
            handlers: Default::default(),
            embedded_resources: Default::default(),
        }
    }

    /// Create a new instance of the custom [`AssetServer`](bevy::asset::AssetServer).
    #[cfg(not(feature = "use-default-assetio"))]
    pub fn new() -> Self {
        BevassetIo {
            handlers: Default::default(),
            embedded_resources: Default::default(),
        }
    }

    /// Add a custom [`AssetServer`](bevy::asset::AssetServer) for handling specific paths.
    pub fn add_handler(&mut self, handler: HandlerConfig) -> &mut Self {
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

async fn load_path_via_handler<'a>(
    path: &'a Path,
    config: &'a HandlerConfig,
    bevasset: &'a BevassetIo,
) -> Result<Vec<u8>, AssetIoError> {
    // first remove the protocol part of the path
    let path = path.display().to_string();
    let path = path
        .strip_prefix(config.protocol.as_str())
        .expect("path does not start with the defined protocol");
    let path = Path::new(path);

    // now load using the handler
    trace!(?path, protocol=?config.protocol, "load asset via handler");
    let r = config.asset_io.load_path(Path::new(path)).await;

    // fallback in case of errors
    match r {
        r @ Ok(_) => {
            trace!(?path, "loaded");
            r
        }
        Err(err) if config.fallback_on_err => {
            info!(?err, ?path, protocol=?config.protocol, "failed loading asset using handler, fallback to default");
            bevasset.load_path(path).await
        }
        Err(err) => {
            warn!(?err, ?path, protocol=?config.protocol, "failed loading asset");
            Err(err)
        }
    }
}

#[cfg(feature = "use-default-assetio")]
async fn load_path<'a>(path: &'a Path, bevasset: &'a BevassetIo) -> Result<Vec<u8>, AssetIoError> {
    if let Some(config) = bevasset
        .handlers
        .iter()
        .find(|h| path.starts_with(h.protocol.as_str()))
    {
        load_path_via_handler(path, config, bevasset).await
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
async fn load_path<'a>(path: &'a Path, bevasset: &'a BevassetIo) -> Result<Vec<u8>, AssetIoError> {
    if let Some(config) = bevasset
        .handlers
        .iter()
        .find(|h| path.starts_with(h.protocol.as_str()))
    {
        load_path_via_handler(path, config, bevasset).await
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
    bevasset: &BevassetIo,
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
impl AssetIo for BevassetIo {
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
            .find(|h| path.starts_with(h.protocol.as_str()))
        {
            // first remove the protocol part of the path
            let path = path.display().to_string();
            let path = path
                .strip_prefix(config.protocol.as_str())
                .expect("path does not start with the defined protocol");
            let path = Path::new(path);
            // pass call to handler
            trace!(?path, protocol=?config.protocol, "read directory via handler");
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
            .find(|h| path.starts_with(h.protocol.as_str()))
        {
            config.asset_io.is_directory(path)
        } else {
            // here there's no chance of doing a fallback.
            // if default_io is enabled, it effectively dictates the result when not using a
            // matched protocol
            self.default_io.is_directory(path)
        };
        is_directory
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.protocol.as_str()))
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
impl AssetIo for BevassetIo {
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
            .find(|h| path.starts_with(h.protocol.as_str()))
        {
            // first remove the protocol part of the path
            let path = path.display().to_string();
            let path = path
                .strip_prefix(config.protocol.as_str())
                .expect("path does not start with the defined protocol");
            let path = Path::new(path);
            // pass call to handler
            trace!(?path, protocol=?config.protocol, "read directory via handler");
            config.asset_io.read_directory(path)
        } else {
            read_embedded_directory(self, path)
        }
    }

    fn is_directory(&self, path: &Path) -> bool {
        let is_directory = if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.protocol.as_str()))
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
            .find(|h| path.starts_with(h.protocol.as_str()))
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

    use super::BevassetIo;

    #[test]
    fn load_path() {
        let mut embedded = BevassetIo::new();
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
        let mut embedded = BevassetIo::new(None);
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
        let mut embedded = BevassetIo::new(None);
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
