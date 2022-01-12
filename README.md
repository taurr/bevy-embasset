# Bevasset-io

TODO: add GH workflow for building & testing --workspace

TODO: update README.md, badges etc.

TODO: add note on usage of LFS and embedding resources via GH workflows

TODO: release on crates.io

[![Unlicense](https://img.shields.io/badge/license-Unlicense-blue)](https://unlicense.org)
[![Release Doc](https://docs.rs/bevasset-io/badge.svg)](https://docs.rs/bevasset-io)
[![Crate](https://img.shields.io/crates/v/bevasset-io.svg)](https://crates.io/crates/bevasset-io)
[![Bevy Tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![CI](https://github.com/vleue/embedded_assets/actions/workflows/ci.yaml/badge.svg)](https://github.com/vleue/embedded_assets/actions/workflows/ci.yaml)

Embed your asset folder inside your binary for easier releases.

Work originally inspired by [bevy_embedded_assets](https://github.com/vleue/bevy_embedded_assets/tree/main/src)

```ignore
use bevy::prelude::*;
use bevasset_io::*;

fn main() {
    App::new()
        .add_bevasset_plugin(add_embedded_assets)
        ...
        .run();
}

// Note: if configuring assets manually, there's no reason for this, nor the build script
include!(concat!(env!("OUT_DIR"), "/add_embedded_assets.rs"));
```

## Bevy Compatibility

|Bevy|bevasset-io|
|---|---|
|0.6|main|
