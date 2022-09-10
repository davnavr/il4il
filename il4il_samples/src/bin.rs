//! Writes examples to disk.

use il4il::module::Module;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Module::from(il4il_samples::return_int("ReturnOk", 0)).write_to_path("return_ok.iil")?;
    Ok(())
}
