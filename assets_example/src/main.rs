use bevasset_io::*;
use bevy::{asset::AssetIo, prelude::*};
use std::path::Path;

fn main() {
    let mut app = App::new();
    app.add_bevasset_plugin(add_embedded_assets).run();

    app.add_bevasset_plugin(|io| {
        // Include all assets, picked up from `build.rs`
        add_embedded_assets(io);

        // configure BevassetIo manually
        io.add_handler(HandlerConfig::new("file", FileAssetIo).fallback_on_err());
        io.add_embedded_asset(Path::new("dummy"), include_bytes!("../assets/.keepme"));
    })
    .run();
}

include!(concat!(env!("OUT_DIR"), "/add_embedded_assets.rs"));

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
