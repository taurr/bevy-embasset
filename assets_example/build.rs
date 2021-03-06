use std::{env, path::Path};

fn main() {
    // Do this to include all files in the asset folder:
    bevy_embasset::include_all_assets(
        &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
    );

    // OR this, just to make sure all your assets are accounted for:
    // if bevy_embasset::include_assets(
    //     &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
    //     &[".keepme"],
    // )
    // .is_err()
    // {
    //     std::process::exit(-1);
    // }
}
