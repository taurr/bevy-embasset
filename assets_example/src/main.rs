use bevy::{asset::AssetIo, prelude::*};
use bevy_embasset::*;
use std::path::Path;

fn main() {
    let mut app = App::new();
    app.add_embasset_plugin(add_embasset_assets).run();

    app.add_embasset_plugin(|io| {
        // Include all assets, picked up from `build.rs`
        add_embasset_assets(io);

        // configure manually
        io.add_handler(AssetIoConfig::new("file", FileAssetIo).fallback_on_err());
        io.add_embedded_asset(Path::new("dummy"), include_bytes!("../assets/.keepme"));
    })
    .run();
}

include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));

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
