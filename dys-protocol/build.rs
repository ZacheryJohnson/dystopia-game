use std::ffi::OsStr;
use std::io::{Result, Write};
use std::path::PathBuf;
use prost_build::{Service, ServiceGenerator};

const BASE_OUTPUT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/generated");
const PROTOS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/protos");

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
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(
            &get_proto_files(),
            &get_proto_includes(),
        )?;

    let mut generated_mod_file = std::fs::File::create(output_dir.join("mod.rs"))?;
    generated_mod_file
        .write_all("#![allow(clippy::all)]\n".as_bytes())
        .expect("failed to write Clippy ignore to generated mod.rs");

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
        for method in &service.methods {
            buf.push_str(
r#"
impl <REQUEST_TYPE> {
    pub fn make_client(&self, nats_client: async_nats::Client) -> <SERVICE_NAME>_svc::<RPC_NAME>RpcClient {
        <SERVICE_NAME>_svc::<RPC_NAME>RpcClient::new(nats_client)
    }
}

#[cfg(feature = "http")]
impl <RESPONSE_TYPE> {
    pub fn to_http(&self) -> crate::http::<PACKAGE_NAME>::<RESPONSE_TYPE> {
        let bytes = postcard::to_allocvec(&self).unwrap();
        postcard::from_bytes(&bytes).unwrap()
    }
}
"#
                .replace("<SERVICE_NAME>", service.name.to_ascii_lowercase().as_str())
                .replace("<RPC_NAME>", method.proto_name.as_str())
                .replace("<REQUEST_TYPE>", method.input_type.as_str())
                .replace("<RESPONSE_TYPE>", method.output_type.as_str())
                .replace("<PACKAGE_NAME>", service.package.as_str())
                .as_str()
            );
        }

        buf.push_str(
            r#"
pub mod <SERVICE_NAME>_svc {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use bytes::Bytes;
    use futures::future::BoxFuture;
    use tower::Service;

    pub(crate) enum HandlerFn<Req, Resp, State> {
        Stateless(Box<dyn Fn(Req) -> BoxFuture<'static, Result<Resp, dys_nats::error::NatsError>> + Send>),
        Stateful(Box<dyn Fn(Req, State) -> BoxFuture<'static, Result<Resp, dys_nats::error::NatsError>> + Send>)
    }

    impl<Req, Resp, State> HandlerFn<Req, Resp, State> {
        pub(crate) fn exec(
            &self,
            request: Req,
            state: Option<State>
        ) -> BoxFuture<'static, Result<Resp, dys_nats::error::NatsError>> {
            match self {
                HandlerFn::Stateless(func) => (func)(request),
                HandlerFn::Stateful(func) => (func)(request, state.unwrap()),
            }
        }
    }
"#
                .replace("<SERVICE_NAME>", service.name.to_ascii_lowercase().as_str())
                .as_str()
        );

        for method in &service.methods {
            buf.push_str(
                r#"
    pub struct <RPC_NAME>RpcClient {
        nats_client: async_nats::Client,
    }

    impl <RPC_NAME>RpcClient {
        pub fn new(nats_client: async_nats::Client) -> Self {
            Self { nats_client }
        }
    }

    impl dys_nats::rpc::client::NatsRpcClient for <RPC_NAME>RpcClient {
        type Request = super::<REQUEST_TYPE>;
        type Response = super::<RESPONSE_TYPE>;

        const RPC_SUBJECT: &'static str = "rpc.<SERVICE_NAME>.<RPC_NAME>";

        fn client(&self) -> async_nats::Client {
            self.nats_client.clone()
        }
    }

    pub struct <RPC_NAME>RpcServer<State> {
        handler_fn: HandlerFn<super::<REQUEST_TYPE>, super::<RESPONSE_TYPE>, State>,
        state: Option<State>,
    }

    impl<State: Clone> dys_nats::rpc::server::NatsRpcServer for <RPC_NAME>RpcServer<State> {
        type Request = super::<REQUEST_TYPE>;
        type Response = super::<RESPONSE_TYPE>;

        const RPC_SUBJECT: &'static str = "rpc.<SERVICE_NAME>.<RPC_NAME>";
    }

    impl<State> <RPC_NAME>RpcServer<State> {
        pub fn with_handler<Func, Fut>(
            handler_fn: Func,
        ) -> <RPC_NAME>RpcServer<State>
        where
            Func: Send + 'static + Fn(super::<REQUEST_TYPE>) -> Fut,
            Fut: Send + 'static + Future<Output = Result<super::<RESPONSE_TYPE>, dys_nats::error::NatsError>>,
        {
            <RPC_NAME>RpcServer {
                handler_fn: HandlerFn::Stateless(Box::new(move |req| Box::pin(handler_fn(req)))),
                state: None,
            }
        }

        pub fn with_handler_and_state<Func, Fut>(
            handler_fn: Func,
            state: State,
        ) -> <RPC_NAME>RpcServer<State>
        where
            Func: Send + 'static + Fn(super::<REQUEST_TYPE>, State) -> Fut,
            Fut: Send + 'static + Future<Output = Result<super::<RESPONSE_TYPE>, dys_nats::error::NatsError>>,

        {
            <RPC_NAME>RpcServer {
                handler_fn: HandlerFn::Stateful(Box::new(move |req, state| Box::pin(handler_fn(req, state)))),
                state: Some(state),
            }
        }
    }

    impl<State: Clone> Service<async_nats::Message> for <RPC_NAME>RpcServer<State> {
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

            let future = self.handler_fn.exec(converted_request, self.state.clone());
            Box::pin(async move {
                let response = future.await;
                match response {
                    Ok(response) => Ok(postcard::to_allocvec(&response).unwrap().into()),
                    Err(err) => Err(err),
                }
            })
        }
    }
"#
                    .replace("<SERVICE_NAME>", service.name.to_ascii_lowercase().as_str())
                    .replace("<RPC_NAME>", method.proto_name.as_str())
                    .replace("<REQUEST_TYPE>", method.input_type.as_str())
                    .replace("<RESPONSE_TYPE>", method.output_type.as_str())
                    .as_str()
            );
        }
        buf.push('}');
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
    generated_mod_file
        .write_all("#![allow(clippy::all)]\n".as_bytes())
        .expect("failed to write Clippy ignore to generated mod.rs");

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

    #[cfg(feature = "http")]
    build_http(output_dir.clone())?;

    #[cfg(feature = "nats")]
    build_nats(output_dir.clone())?;

    Ok(())
}