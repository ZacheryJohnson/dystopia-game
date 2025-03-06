use std::ffi::OsStr;
use std::io::{Result, Write};
use std::path::PathBuf;

use walkdir;

const OUTPUT_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/generated");
const PROTOS_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/protos");

fn get_proto_files() -> Vec<String> {
    let mut proto_files = vec![];

    for dir_entry in walkdir::WalkDir::new(PROTOS_DIR) {
        let Ok(dir_entry) = dir_entry else {
            continue
        };

        if dir_entry.file_type().is_file() {
            let file_path = dir_entry.path();
            if file_path.extension() == Some(OsStr::new("proto")) {
                proto_files.push(file_path.display().to_string());
            }
        }
    }

    proto_files
}

fn get_proto_includes() -> Vec<String> {
    let includes = vec![String::from(PROTOS_DIR)];

    includes
}

fn main() -> Result<()> {
    let output_dir = PathBuf::from(OUTPUT_DIR);

    // Empty any existing generated files
    if std::fs::exists(output_dir.clone())? {
        std::fs::remove_dir_all(output_dir.clone())?;
        std::fs::create_dir(output_dir.clone())?;
    }

    tonic_build::configure()
        .out_dir(output_dir.clone())
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .compile_protos(
            &get_proto_files(),
            &get_proto_includes(),
        )?;

    let mut generated_mod_file = std::fs::File::create(output_dir.join("mod.rs"))?;
    for generated_proto_file in walkdir::WalkDir::new(OUTPUT_DIR) {
        let Ok(generated_proto_file) = generated_proto_file else {
            continue;
        };

        if generated_proto_file.file_type().is_dir() {
            continue;
        }

        let file_name_without_ext = generated_proto_file
            .file_name()
            .to_str()
            .unwrap()
            .strip_suffix(".rs")
            .unwrap();

        if file_name_without_ext == "mod" {
            continue;
        }

        let mod_name = format!("pub mod {file_name_without_ext};\n");
        generated_mod_file.write_all(mod_name.as_bytes())?;
    }

    Ok(())
}