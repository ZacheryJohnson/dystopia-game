fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = "./generated";

    tonic_build::configure()
        .out_dir(out_dir)
        .build_client(true)
        .build_server(false)
        .compile(&["../protocol/Director.proto"], &["../protocol"])?;

    Ok(())
}