# Bevy-Embasset

[![Crate](https://img.shields.io/crates/v/bevy-embasset.svg)](https://crates.io/crates/bevy-embasset)
[![Release Doc](https://docs.rs/bevy-embasset/badge.svg)](https://docs.rs/bevy-embasset)
[![Test Status](https://github.com/taurr/bevy-embasset/actions/workflows/build_n_test.yml/badge.svg)](https://github.com/taurr/bevy-embasset/actions)

Embed your asset folder inside your binary.

`bevy-embasset` adds support for loading assets embedded into the binary.

Furthermore, it can optionally try to load assets via the default [`AssetPlugin`](bevy::asset::AssetPlugin)
first, thereby allowing the embedded assets to be used as fallbacks in case of problems.

As icing on the cake, `bevy-embasset` allows to register multiple other
[`AssetServer`](bevy::asset::AssetServer)'s, that will be used for asset paths beginning with
specific configurable strings. This can be used to have some assets load from e.g. a web-service,
while others are loaded from disk or embedded in the binary. It can also be used to e.g. build
1 or more sub-crates each using [`EmbassetIo`](EmbassetIo) and holding a set of assets - thereby
saving compile-time as the assets don't have to be compiled every time (An example of this, can be found in the
game [`bevoids`](https://github.com/taurr/bevoids)).

# Usage

### Recommended

An easy way of adding an enum with some assets, an `AssetIo` handling those assets, and adding the
`EmbassetIo` that can handle all the loading/routing is:

```rust
use bevy::{prelude::*, asset::AssetPlugin};
use bevy_embasset::{EmbassetPlugin, embasset_assets};

embasset_assets!(
    pub enum GameAssets {
        #[doc = "It's possible to document each enum variant"]
        Icon = "icon.png",
        BackgroundMusic = "sounds/bg.wav"
    },
    pub struct GameAssetsIo {
        root = "../assets/"
    }
);

fn main() {
    App::new().add_plugins_with(DefaultPlugins, |group| {
        group.add_before::<AssetPlugin, _>(EmbassetPlugin::new(|io| {
            io.add_handler(GameAssetsIo::new().into());
        }))
    });
}
```

Note, in the above example, `GameAssets` and `GameAssetsIo` (defined by the macro) can be moved to a
separate crate - saving compile time.

The defined `GameAssets` implements several useful standard traits: `Eq`, `Ord`, `Hash`, `Copy`.
It can be iterated over by invoking `GameAssets::iter()`, and the number of assets is available in 
`GameAssets::COUNT`.

The path needed to load an asset through the added `EmbassetPlugin` is retrieved using e.g. 
`GameAssets::Icon.path()`:

```rust
fn some_asset_loading_system(asset_server: &AssetServer) {
  let icon : Handle<Image> = asset_server.load(GameAssets::Icon.path());
}
```

### Using `build.rs`, no identifying enum

```rust
use bevy::{prelude::*, asset::AssetPlugin};
use bevy_embasset::AddEmbassetPlugin;

```rust
use bevy::{prelude::*, asset::AssetPlugin};
use bevy_embasset::EmbassetPlugin;

fn main() {
    App::new().add_plugins_with(DefaultPlugins, |group| {
        group.add_before::<AssetPlugin, _>(EmbassetPlugin::new(add_embasset_assets))
    });
}
```

Generate the `add_embasset_assets` function from a build script (`build.rs`):

```rust
use std::{env, path::Path};

fn main() {
    // Do this to include all files in the asset folder:
    bevy_embasset::include_all_assets(
        &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
    );
}
```

and included it in the source:

```rust
include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));
```

For the build script, the feature `build` needs to be enabled in `Cargo.toml`:

```toml
[build-dependencies]
bevy-embasset = { version = "*", features = ["build"] }
```

## Bevy Compatibility

|bevy-embasset|Bevy|
|-------------|----|
|main         |0.6 |
