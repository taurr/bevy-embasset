use bevy::prelude::*;
use bevy_embasset::*;
use std::path::Path;

assets!(
    pub enum GameAssets {
        #[doc = "Dummy documentation"]
        Icon = ".keepme",
    },
    pub struct GameAssetsIo {
        root = "../assets/"
    }
);

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
        io.add_handler(GameAssetsIo::new().into());
    })
    .run();
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPath;
    use bevy_embasset::EnumCount;

    use super::*;

    #[test]
    fn feature() {
        assert_eq!(1, GameAssets::COUNT);
    }

    #[test]
    fn game_assets_io_is_asset_io_alternative() {
        fn assert<T: Into<AssetIoAlternative>>() {}
        assert::<GameAssetsIo>();
    }

    #[test]
    fn game_assets_is_asset_path() {
        fn assert<'a, T: 'a + Into<AssetPath<'a>>>() {}
        assert::<GameAssets>();
    }
}
