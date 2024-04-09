fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/project_budget.proto")?;
    capnpc::CompilerCommand::new()
        .src_prefix("proto")
        .file("proto/project_budget.capnp")
        .run()?;
    Ok(())
}
