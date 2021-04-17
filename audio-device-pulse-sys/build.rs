fn main() -> anyhow::Result<()> {
    pkg_config::Config::new().statik(false).probe("libpulse")?;
    Ok(())
}
