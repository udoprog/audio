use anyhow::anyhow;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let lib = pkg_config::Config::new()
        .statik(false)
        .cargo_metadata(false)
        .probe("libpipewire-0.3")?;
    generate_bindings(&lib)?;
    Ok(())
}

fn generate_bindings(lib: &pkg_config::Library) -> anyhow::Result<()> {
    let include_args = lib.include_paths.iter().map(|include_path| {
        format!(
            "-I{}",
            include_path.to_str().expect("include path was not UTF-8")
        )
    });

    let mut config = bindgen::CodegenConfig::empty();
    config.insert(bindgen::CodegenConfig::FUNCTIONS);
    config.insert(bindgen::CodegenConfig::TYPES);

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let output = root
        .join("..")
        .join("audio-device-pipewire-sys")
        .join("src")
        .join("bindings.rs");

    let builder = bindgen::Builder::default()
        .size_t_is_usize(true)
        .allowlist_recursively(false)
        .prepend_enum_name(false)
        .layout_tests(false)
        .allowlist_function("pw_.*")
        .allowlist_type("pw_.*")
        .allowlist_function("spa_.*")
        .allowlist_type("spa_.*")
        .allowlist_type("__va_list_.*")
        .with_codegen_config(config)
        .clang_args(include_args)
        .header(root.join("pipewire.h").display().to_string());

    let bindings = builder
        .generate()
        .map_err(|()| anyhow!("Unable to generate bindings"))?;

    bindings.write_to_file(output)?;
    Ok(())
}
