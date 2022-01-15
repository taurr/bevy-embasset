use bevy::prelude::*;
use bevy_embasset::*;

fn main() {
    let mut app = App::new();

    // add embasset as a plugin, include assets as found by the build script
    app.add_embasset_plugin(add_embasset_assets).run();
}

include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));
