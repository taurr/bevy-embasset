#[cfg(feature = "use-default-assetio")]
use bevy::asset::create_platform_default_asset_io;
use bevy::{
    prelude::{App, AssetServer, Plugin},
    tasks::IoTaskPool,
};

use crate::EmbassetIo;

/// Bevy plugin that will insert [`EmbassetIo`](EmbassetIo) instead of the default
/// [`AssetServer`](bevy::asset::AssetServer) added by the [`AssetPlugin`](bevy::asset::AssetPlugin).
///
/// # Examples
///
/// If you are using the [`DefaultPlugins`](bevy::prelude::DefaultPlugins) group from Bevy, it can
/// be added a couple of different ways.
///
/// ## Add Embasset Plugin
///
/// Without [`DefaultPlugins`](bevy::prelude::DefaultPlugins) from Bevy, it's a simple matter of
/// adding the plugin:
///
/// ```rust
/// # use bevy::{prelude::*, asset::AssetPlugin};
/// # use bevy_embasset::AddEmbassetPlugin;
/// # fn main() {
///  App::new().add_plugin(EmbassetPlugin::new(add_embasset_assets));
/// # }
/// # fn add_embasset_assets(_: &mut bevy_embasset::EmbassetIo){}
/// ```
///
/// If however, `DefaultPlugins` are used, bevy inserts its `AssetPlugin` which causes
/// a little extra work in order to add the plugin:
///
/// ```rust
/// # use bevy::{prelude::*, asset::AssetPlugin};
/// # use bevy_embasset::AddEmbassetPlugin;
/// # fn main() {
///  App::new().add_embasset_plugin(add_embasset_assets);
/// # }
/// # fn add_embasset_assets(_: &mut bevy_embasset::EmbassetIo) {}
/// ```
///
/// Or a little more manually:
///
/// ```rust
/// # use bevy::{prelude::*, asset::AssetPlugin};
/// # use bevy_embasset::EmbassetPlugin;
/// # fn main() {
///  App::new().add_plugins_with(DefaultPlugins, |group| {
///      group.add_before::<AssetPlugin, _>(EmbassetPlugin::new(add_embasset_assets))
///  });
/// # }
/// # fn add_embasset_assets(_: &mut bevy_embasset::EmbassetIo){
/// # }
/// ```
///
/// ## Configure the plugin
///
/// The `add_embasset_assets` function from above can be generated from a build script (`build.rs`):
///
/// ```ignore
/// bevy_embasset::include_all_assets(
///  &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets")
/// );
/// ```
///
/// and includeded in the source:
///
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));
/// ```
///
/// Alternatively, the `add_embasset_assets` can be created manually, gaining the ultimate flexibility:
///
/// ```ignore
/// fn add_embasset_assets(io: &mut EmbassetIo){
///     // add assets manually...
///     io.add_embedded_asset(Path::new("dummy"), include_bytes!("../assets/.keepme"));
///
///     // or add other custom, AssetIo - for handling specific paths...
///     io.add_handler(AssetIoAlternative::new("dummy://", DummyAssetIo).fallback_on_err());
/// }
/// ```
///
/// To use the build script, the `build` feature needs to be enabled when building it:
///
/// ```toml
/// [build-dependencies]
/// bevy-embasset = { version = "*", features = ["build"] }
/// ```
///
#[derive(Debug)]
pub struct EmbassetPlugin<F> {
    initializer: F,
}

impl<F> EmbassetPlugin<F>
where
    F: Fn(&mut EmbassetIo) + Send + Sync + 'static,
{
    /// Create a new instance of the plugin.
    pub fn new(asset_initializer: F) -> Self {
        Self {
            initializer: asset_initializer,
        }
    }
}

impl<F> Plugin for EmbassetPlugin<F>
where
    F: Fn(&mut EmbassetIo) + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        #[cfg(feature = "use-default-assetio")]
        let mut bevy_embasset =
            EmbassetIo::with_default_assetio(create_platform_default_asset_io(app));

        #[cfg(not(feature = "use-default-assetio"))]
        let mut bevy_embasset = EmbassetIo::new();

        let initializer = &self.initializer;
        initializer(&mut bevy_embasset);

        let task_pool = app
            .world
            .get_resource::<IoTaskPool>()
            .expect("`IoTaskPool` resource not found.")
            .0
            .clone();

        app.insert_resource(AssetServer::new(bevy_embasset, task_pool));
    }
}
