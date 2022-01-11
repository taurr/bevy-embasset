use std::{env, path::Path};

fn main() {
    bevasset_io::generate_include_all_assets(
        &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets")
    );
}
