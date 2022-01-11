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
use derive_more::DebugCustom;
pub use plugin::BevassetIoPlugin;

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::generate_include_all_assets;

use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture},
    prelude::debug,
};
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
    fallthrough: bool,
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
    /// - **fallthrough**
    ///
    ///     Whether `BevassetIo` should attempt to load the asset from it's embedded resources in case
    /// of  an error from the handler.
    ///
    /// - **asset_io**
    ///
    ///     Handler for loading an asset using the specified `protocol`.
    pub fn new<T: AssetIo>(protocol: &str, fallthrough: bool, asset_io: T) -> Self {
        HandlerConfig {
            protocol: SmolStr::new(protocol),
            fallthrough,
            asset_io: Box::new(asset_io),
        }
    }
}

/// Custom [`AssetServer`](bevy::asset::AssetServer), that can load assets embedded into the binary,
/// or use other servers for handling the load.
#[derive(Debug)]
pub struct BevassetIo {
    handlers: Vec<HandlerConfig>,
    embedded_resources: HashMap<&'static Path, &'static [u8]>,
}

impl Default for BevassetIo {
    fn default() -> Self {
        BevassetIo::new()
    }
}

impl BevassetIo {
    /// Create a new instance of the custom [`AssetServer`](bevy::asset::AssetServer).
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

impl AssetIo for BevassetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        if let Some(_config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.protocol.as_str()))
        {
            todo!();
            // How do we remove protocol from the start of path, then return
            // a) asset_io.load_path(new_path)
            // or b) a new future that handles the error case of am by invoking this very method with the new_path
        } else {
            debug!(?path, "loaded as embedded resource");
            Box::pin(async move {
                self.embedded_resources
                    .get(path)
                    .map(|b| b.to_vec())
                    .ok_or_else(|| bevy::asset::AssetIoError::NotFound(path.to_path_buf()))
            })
        }
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        // TODO: follow pattern from load_path
        if self.is_directory(path) {
            #[allow(clippy::needless_collect)]
            let paths: Vec<_> = self
                .embedded_resources
                .keys()
                .filter(|loaded_path| loaded_path.starts_with(path))
                .map(|t| t.to_path_buf())
                .collect();
            Ok(Box::new(paths.into_iter()))
        } else {
            Err(AssetIoError::Io(std::io::ErrorKind::NotFound.into()))
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
        debug!(?path, ?is_directory);
        is_directory
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        if let Some(config) = self
            .handlers
            .iter()
            .find(|h| path.starts_with(h.protocol.as_str()))
        {
            debug!(?config.protocol, ?path, "off-handling path watching");
            config.asset_io.watch_path_for_changes(path)
        } else {
            debug!(?path, "not really watching!");
            Ok(())
        }
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        debug!("not really watching!");
        Ok(())
    }
}

#[cfg(test)]
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
        let mut embedded = BevassetIo::new();
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
        let mut embedded = BevassetIo::new();
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
