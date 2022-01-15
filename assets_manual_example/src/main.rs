use bevy::{asset::AssetIo, prelude::*};
use bevy_embasset::*;
use std::path::{Path, PathBuf};

fn main() {
    let mut app = App::new();

    app.add_embasset_plugin(|io| {
        // configure manually
        // Add an embedded asset
        io.add_embedded_asset(Path::new("dummy"), include_bytes!("../assets/.keepme"));

        // Add another, custom, AssetIo - for handling all paths starting with "dummy://"
        // If this AssetIo fails, Bevasset will try other means...
        // If Bevasset has been compiled with the "use-default-assetio" feature, Bevasset will
        // attempt to load all assets through Bevy's default AssetIo before reverting to the
        // embedded resources. Otherwise we just use the embedded resources.
        io.add_handler(AssetIoAlternative::new("dummy://", DummyAssetIo).fallback_on_err());
    })
    .run();
}

struct DummyAssetIo;

impl AssetIo for DummyAssetIo {
    fn load_path<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::asset::BoxedFuture<'a, Result<Vec<u8>, bevy::asset::AssetIoError>> {
        Box::pin(async move { Err(bevy::asset::AssetIoError::NotFound(PathBuf::from(path))) })
    }

    fn read_directory(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn Iterator<Item = std::path::PathBuf>>, bevy::asset::AssetIoError> {
        Err(bevy::asset::AssetIoError::NotFound(PathBuf::from(path)))
    }

    fn is_directory(&self, _path: &std::path::Path) -> bool {
        false
    }

    fn watch_path_for_changes(
        &self,
        _path: &std::path::Path,
    ) -> Result<(), bevy::asset::AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), bevy::asset::AssetIoError> {
        Ok(())
    }
}
