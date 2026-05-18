fn main() -> Result<(), serde_json::Error> {
    serde_json::to_writer_pretty(std::io::stdout(), &vatprc_uniapi::openapi::openapi())?;
    println!();
    Ok(())
}
