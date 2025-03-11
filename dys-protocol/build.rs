use std::ffi::OsStr;
use std::io::{Result, Write};
use std::path::PathBuf;
use prost_build::{Service, ServiceGenerator};
use walkdir;

const BASE_OUTPUT_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/generated");
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

fn build_http(output_dir: PathBuf) -> Result<()> {
    let output_dir = output_dir.join("http");

    // Empty any existing generated files
    if std::fs::exists(output_dir.clone())? {
        std::fs::remove_dir_all(output_dir.clone())?;
    }
    std::fs::create_dir(output_dir.clone())?;

    tonic_build::configure()
        .out_dir(output_dir.clone())
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .type_attribute(".", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile_protos(
            &get_proto_files(),
            &get_proto_includes(),
        )?;

    let mut generated_mod_file = std::fs::File::create(output_dir.join("mod.rs"))?;
    for generated_proto_file in walkdir::WalkDir::new(output_dir) {
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

struct NatsServiceGenerator;

impl ServiceGenerator for NatsServiceGenerator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        buf.push_str(
r#"
pub mod <SERVICE_NAME>_svc {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use bytes::Bytes;
    use futures::future::BoxFuture;
    use futures::FutureExt;
    use tower::Service;

"#
            .replace("<SERVICE_NAME>", service.name.to_ascii_lowercase().as_str())
            .as_str()
        );

        for method in service.methods {
            buf.push_str(
r#"
    pub struct <RPC_NAME>RpcService {
        handler_fn: Box<dyn Fn(super::<REQUEST_TYPE>) -> BoxFuture<'static, Result<super::<RESPONSE_TYPE>, dys_nats::error::NatsError>>>,
    }
    impl <RPC_NAME>RpcService {
        pub fn with_handler<Func, Fut>(
            handler_fn: Func,
        ) -> <RPC_NAME>RpcService
        where
            Func: Send + 'static + Fn(super::<REQUEST_TYPE>) -> Fut,
            Fut: Send + 'static + Future<Output = Result<super::<RESPONSE_TYPE>, dys_nats::error::NatsError>>,
        {
            <RPC_NAME>RpcService {
                handler_fn: Box::new(move |req| Box::pin(handler_fn(req))),
            }
        }
    }

    impl Service<async_nats::Message> for <RPC_NAME>RpcService {
        type Response = Bytes;
        type Error = dys_nats::error::NatsError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: async_nats::Message) -> Self::Future {
            let Ok(converted_request) = postcard::from_bytes(&req.payload.to_vec()) else {
                return Box::pin(async { Err(dys_nats::error::NatsError::MalformedRequest) });
            };

            let future = (self.handler_fn)(converted_request);
            Box::pin(async move {
                let response = future.await;
                let Ok(response_bytes) = postcard::to_allocvec(&response) else {
                    return Err(dys_nats::error::NatsError::InternalSerializationError);
                };
                Ok(response_bytes.to_vec().into())
            })
        }
    }
"#
                .replace("<RPC_NAME>", method.proto_name.as_str())
                .replace("<REQUEST_TYPE>", method.input_type.as_str())
                .replace("<RESPONSE_TYPE>", method.output_type.as_str())
                .as_str()
            );
        }
        buf.push_str("}");

        println!("{buf}");
    }
}

fn build_nats(output_dir: PathBuf) -> Result<()> {
    let output_dir = output_dir.join("nats");

    // Empty any existing generated files
    if std::fs::exists(output_dir.clone())? {
        std::fs::remove_dir_all(output_dir.clone())?;
    }
    std::fs::create_dir(output_dir.clone())?;

    prost_build::Config::new()
        .out_dir(output_dir.clone())
        .type_attribute(".", "#[derive(serde::Deserialize, serde::Serialize)]")
        .service_generator(Box::new(NatsServiceGenerator))
        .compile_protos(
            &get_proto_files(),
            &get_proto_includes(),
        )?;

    let mut generated_mod_file = std::fs::File::create(output_dir.join("mod.rs"))?;
    for generated_proto_file in walkdir::WalkDir::new(output_dir) {
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

fn main() -> Result<()> {
    let output_dir = PathBuf::from(BASE_OUTPUT_DIR);

    // Empty any existing generated files
    if std::fs::exists(output_dir.clone())? {
        std::fs::remove_dir_all(output_dir.clone())?;
    }
    std::fs::create_dir(output_dir.clone())?;

    build_http(output_dir.clone())?;
    build_nats(output_dir.clone())?;

    Ok(())
}