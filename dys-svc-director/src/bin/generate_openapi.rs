use utoipa::OpenApi;
use director::DirectorApi;

fn main() {
    println!("Generating OpenAPI spec...");
    let doc = gen_openapi();

    println!("Creating output dir...");
    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/generated/openapi.json");
    std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/generated")).expect("failed to create output directory");

    println!("Writing spec to file...");
    std::fs::write(output_path, doc).expect("failed to write OpenAPI spec");
}

fn gen_openapi() -> String {
    DirectorApi::openapi().to_pretty_json().unwrap()
}