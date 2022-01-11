use bevasset_io::{BevassetIoPlugin, HandlerConfig};
use bevy::{
    asset::{AssetIo, AssetPlugin},
    prelude::*,
};
use std::path::Path;

fn main() {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |group| {
        group.add_before::<AssetPlugin, _>(BevassetIoPlugin::new(|io| {
            // Include all assets, picked up from `build.rs`
            include_all_assets(io);
            // configure BevassetIo manually
            io.add_handler(HandlerConfig::new("file", true, FileAssetIo))
                .add_embedded_asset(Path::new("dummy"), include_bytes!("../assets/.keepme"));
        }))
    })
    .run();
}

include!(concat!(env!("OUT_DIR"), "/include_all_assets.rs"));

struct FileAssetIo;

impl AssetIo for FileAssetIo {
    fn load_path<'a>(
        &'a self,
        _path: &'a std::path::Path,
    ) -> bevy::asset::BoxedFuture<'a, Result<Vec<u8>, bevy::asset::AssetIoError>> {
        todo!()
    }

    fn read_directory(
        &self,
        _path: &std::path::Path,
    ) -> Result<Box<dyn Iterator<Item = std::path::PathBuf>>, bevy::asset::AssetIoError> {
        todo!()
    }

    fn is_directory(&self, _path: &std::path::Path) -> bool {
        todo!()
    }

    fn watch_path_for_changes(
        &self,
        _path: &std::path::Path,
    ) -> Result<(), bevy::asset::AssetIoError> {
        todo!()
    }

    fn watch_for_changes(&self) -> Result<(), bevy::asset::AssetIoError> {
        todo!()
    }
}
