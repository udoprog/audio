use anyhow::{anyhow, Result};

pub fn main() -> Result<()> {
    let output = rotary_device::directsound::default_output_device()?;
    let output = output.ok_or_else(|| anyhow!("no default device found"));

    Ok(())
}
