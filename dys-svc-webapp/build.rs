use std::{ffi::OsString, str::FromStr};

const MATCH_VISUALIZER_WASM_DIR_LOCAL_PATH: &'static str = "../.wasm_out";
const FRONTEND_LOCAL_DIR_NAME: &'static str = "frontend";
const FRONTEND_ARTIFACT_PUBLIC_DIR_NAME: &'static str = "public";
const FRONTEND_ARTIFACT_INTERNAL_DIR_NAME: &'static str = "src/assets";

fn build_and_copy_wasm(project_dir_path: &String) {
    let match_visualizer_wasm_path = std::path::Path::new(&project_dir_path).join(MATCH_VISUALIZER_WASM_DIR_LOCAL_PATH);
    let maybe_match_visualizer_wasm_dir = std::fs::read_dir(match_visualizer_wasm_path);
    if maybe_match_visualizer_wasm_dir.is_err() {
        panic!("Failed to read match visualizer WASM dir");
    }

    for maybe_dir_entry in maybe_match_visualizer_wasm_dir.unwrap() {
        if maybe_dir_entry.is_err() {
            panic!("Failed to read dir entry in match visualizer WASM dir");
        }

        let dir_entry = maybe_dir_entry.unwrap();

        let to_dir = {
            // The wasm file should be served publicly; everything else will be internal
            if dir_entry.file_name() == OsString::from_str("matchvisualizer_opt.wasm").unwrap() {
                format!("{project_dir_path}/{FRONTEND_LOCAL_DIR_NAME}/{FRONTEND_ARTIFACT_PUBLIC_DIR_NAME}")
            } else {
                format!("{project_dir_path}/{FRONTEND_LOCAL_DIR_NAME}/{FRONTEND_ARTIFACT_INTERNAL_DIR_NAME}")
            }
        };
        let _ = std::fs::copy(
            dir_entry.path(), 
            format!("{to_dir}/{}", dir_entry.file_name().into_string().unwrap())
        ).expect("failed to copy match visualizer artifact into distribution directory");
    }
}

fn exec_build_script_cmd(project_dir_path: &String) {
    #[cfg(target_os="windows")]
    {
        const BUILD_SCRIPT_CMD_PWSH_LOCAL_PATH: &'static str = "build_scripts/build_webapp_frontend.ps1";
        let _ = std::process::Command::new("powershell")
            .arg("-File")
            .arg(BUILD_SCRIPT_CMD_PWSH_LOCAL_PATH)
            .current_dir(project_dir_path)
            .output()
            .unwrap();
    }

    
    #[cfg(target_os="linux")]
    {
        const BUILD_SCRIPT_CMD_SH_LOCAL_PATH: &'static str = "build_scripts/build_webapp_frontend.sh";
        let _ = std::process::Command::new("sh")
            .arg(BUILD_SCRIPT_CMD_SH_LOCAL_PATH)
            .current_dir(project_dir_path)
            .output()
            .unwrap();
    }
}

fn main() {
    let project_dir_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    build_and_copy_wasm(&project_dir_path);    
    exec_build_script_cmd(&project_dir_path);
}