# bevy-embasset

TODO: use bevy-embasset in bevoids

TODO: add GH workflow for building & testing --workspace

TODO: update README.md, badges etc.

TODO: add note on usage of LFS and embedding resources via GH workflows

TODO: release on crates.io

[![Unlicense](https://img.shields.io/badge/license-Unlicense-blue)](https://unlicense.org)
[![Release Doc](https://docs.rs/bevy-embasset/badge.svg)](https://docs.rs/bevy-embasset)
[![Crate](https://img.shields.io/crates/v/bevy-embasset.svg)](https://crates.io/crates/bevy-embasset)
[![Bevy Tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![CI](https://github.com/vleue/embedded_assets/actions/workflows/ci.yaml/badge.svg)](https://github.com/vleue/embedded_assets/actions/workflows/ci.yaml)

Embed your asset folder inside your binary for easier releases.

Work originally inspired by [bevy_embedded_assets](https://github.com/vleue/bevy_embedded_assets/tree/main/src)

```ignore
use bevy::prelude::*;
use bevy_embasset::*;

fn main() {
    App::new()
        .add_embasset_plugin(add_embasset_assets)
        ...
        .run();
}

// Note: if configuring assets manually, there's no reason for this, nor the build script
include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));
```

## Bevy Compatibility

|Bevy|bevy-embasset|
|---|---|
|0.6|main|
