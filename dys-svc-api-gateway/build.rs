use utoipa::OpenApi;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    gather_openapi_specs();

    Ok(())
}

fn gather_openapi_specs() {
    let walkdir = walkdir::WalkDir::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/..")
    ).max_depth(1);

    let mut supported_service_names = vec![];
    for maybe_entry in walkdir {
        let Ok(entry) = maybe_entry else {
            continue;
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        if entry.file_name().to_str().unwrap().starts_with("dys-svc-") {
            supported_service_names.push(
                entry.file_name().to_str().unwrap().to_string(),
            );
        }
    }

    #[derive(utoipa::OpenApi)]
    struct OpenApi;
    let mut api = OpenApi::openapi();

    for service_name in supported_service_names {
        let walkdir = walkdir::WalkDir::new(format!("../{service_name}/generated"));
        for entry in walkdir {
            let Ok(entry) = entry else {
                continue;
            };

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

    std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/generated")).expect("failed to create output directory");
    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/generated/openapi.json"),
        api.to_pretty_json().unwrap(),
    ).unwrap();
}