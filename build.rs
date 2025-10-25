fn main() -> Result<(), Box<dyn std::error::Error>> {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/message.capnp")
        .run()?;
    Ok(())
}
