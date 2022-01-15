use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/// Generate a function for including *all* assets from a folder in [`EmbassetIo`](crate::EmbassetIo).
///
/// For use from a build script (`build.rs`).
///
/// Signature of the generated function:
///
/// ```ignore
/// fn add_embasset_assets(_: &mut bevy_embasset::EmbassetIo) {
///     ...
/// }
/// ```
///
/// # Requires
///
/// Feature: `build`
///
pub fn include_all_assets(asset_folder: &Path) {
    let method_name = "add_embasset_assets";
    let mut output_file =
        File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join("add_embasset_assets.rs"))
            .unwrap();

    println!("cargo:rerun-if-changed={}", asset_folder.display());

    output_file
        .write_all(
            format!(
                "fn {}(#[allow(unused)] in_memory: &mut bevy_embasset::EmbassetIo){{\n",
                method_name
            )
            .as_ref(),
        )
        .unwrap();
    visit_dirs(asset_folder)
        .iter()
        .map(|path| (path, path.strip_prefix(asset_folder).unwrap()))
        .for_each(|(fullpath, path)| {
            output_file.write_all(
                format!(
                    "    in_memory.add_embedded_asset(std::path::Path::new({:?}), include_bytes!({:?}));",
                    path.to_string_lossy(),
                    fullpath.to_string_lossy()
                )
                .as_ref(),
            )
            .unwrap();
        });
    output_file.write_all("}".as_ref()).unwrap();
}

/// Generate a function for including specific assets in [`EmbassetIo`](crate::EmbassetIo).
///
/// For use from a build script (`build.rs`).
///
/// Signature of the generated function:
///
/// ```ignore
/// fn add_embasset_assets(_: &mut bevy_embasset::EmbassetIo) {
///     ...
/// }
/// ```
///
/// # Requires
///
/// Feature: `build`
///
pub fn include_assets(asset_folder: &Path, assets: &[&str]) -> Result<(), String> {
    let method_name = "add_embasset_assets";
    let mut output_file =
        File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join("add_embasset_assets.rs"))
            .unwrap();

    println!("cargo:rerun-if-changed={}", asset_folder.display());

    output_file
        .write_all(
            format!(
                "fn {}(#[allow(unused)] in_memory: &mut bevy_embasset::EmbassetIo){{\n",
                method_name
            )
            .as_ref(),
        )
        .unwrap();

    for asset in assets {
        let path = asset_folder.join(asset);
        if !path.exists() {
            let err = format!("Asset not found: {}", path.display());
            println!("cargo:warning={}", err);
            return Err(err);
        }
        output_file.write_all(
            format!(
                "    in_memory.add_embedded_asset(std::path::Path::new({:?}), include_bytes!({:?}));",
                asset,
                path.to_string_lossy()
            )
            .as_ref(),
        )
        .unwrap();
    }

    output_file.write_all("}".as_ref()).unwrap();

    Ok(())
}

fn visit_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut collected = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collected.append(&mut visit_dirs(&path));
            } else {
                collected.push(path);
            }
        }
    }
    collected
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn visit_src_dirs() {
        let result = visit_dirs(Path::new("./assets_example"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./assets_example/build.rs"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./assets_example/src/main.rs"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./assets_example/assets/.keepme"));
    }
}
