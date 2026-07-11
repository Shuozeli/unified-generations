fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: Build scripts run in a single-purpose process before compilation.
    // Setting PROTOC here only affects this crate's prost/tonic codegen step.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    tonic_build::compile_protos("proto/doubao/agentplan/v1/agent_plan.proto")?;
    Ok(())
}
