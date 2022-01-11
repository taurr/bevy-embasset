use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/// Generated a function for including assets in [`BevassetIo`](bevasset_io::BevassetIo).
///
/// ```ignore
/// fn include_all_assets(in_memory: &mut bevasset_io::BevassetIo) {
///     ...
/// }
/// ```
///
pub fn generate_include_all_assets(asset_folder: &Path) {
    let method_name = "include_all_assets";
    let mut output_file =
        File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join("include_all_assets.rs"))
            .unwrap();

    output_file
        .write_all(
            format!(
                "fn {}(#[allow(unused)] in_memory: &mut bevasset_io::BevassetIo){{\n",
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
    println!("cargo:rerun-if-changed={}", asset_folder.display());
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
        let result = visit_dirs(Path::new("./example"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./example/build.rs"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./example/src/main.rs"));
        assert!(result
            .iter()
            .any(|p| p.display().to_string() == *"./example/assets/.keepme"));
    }
}
