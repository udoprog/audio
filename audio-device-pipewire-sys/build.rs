fn main() -> anyhow::Result<()> {
    pkg_config::Config::new()
        .statik(false)
        .probe("libpipewire-0.3")?;
    Ok(())
}
