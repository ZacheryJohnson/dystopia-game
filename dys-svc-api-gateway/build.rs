use utoipa::OpenApi;

fn main() -> std::io::Result<()>  {
    gather_openapi_specs();

    Ok(())
}

fn gather_openapi_specs() {
    let toml_file = std::fs::read_to_string("./Cargo.toml").unwrap();
    let toml_contents = toml_file.split_ascii_whitespace();
    let supported_service_names = toml_contents
        .filter(|word| word.starts_with("dys-svc-"))
        .collect::<Vec<_>>();


    #[derive(utoipa::OpenApi)]
    struct OpenApi;
    let mut api = OpenApi::openapi();

    for service_name in supported_service_names {
        // let mut build_process = std::process::Command::new("cargo")
        //     .args(["run", "--release", "-p", service_name, "--bin", "gen-openapi"])
        //     .current_dir(env!("CARGO_MANIFEST_DIR"))
        //     .stdout(std::process::Stdio::null())
        //     .stderr(std::process::Stdio::null())
        //     .spawn()
        //     .unwrap();
        //
        // build_process.wait().unwrap();

        let walkdir = walkdir::WalkDir::new(format!("../{service_name}/generated"));
        for entry in walkdir {
            let entry = entry.unwrap();
            if !entry.file_type().is_file() {
                continue;
            }

            if entry.file_name().to_str().unwrap().starts_with("openapi") {
                let svc_spec_str = std::fs::read_to_string(entry.path()).unwrap();
                let svc_spec: utoipa::openapi::OpenApi = serde_json::from_str(&svc_spec_str).unwrap();

                api.merge(svc_spec);
            }
        }
    }

    api.info.title = "Dystopia API".to_string();

    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/generated/openapi.json"),
        api.to_pretty_json().unwrap(),
    ).unwrap();
}