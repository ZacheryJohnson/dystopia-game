use std::collections::HashMap;
use std::fmt::Display;
use std::time::Duration;
use async_nats::HeaderMap;
use axum::extract::Request;
use axum::http::Response;
use axum::response::IntoResponse;
use futures::StreamExt;
use serde_json::json;
use tower::service_fn;
use utoipa::openapi::{HttpMethod, Object, RefOr, Schema, Type};
use utoipa::openapi::path::Operation;
use utoipa::openapi::schema::SchemaType;
use utoipa_swagger_ui::SwaggerUi;
use bytes::Bytes;
use dys_nats::connection::{make_client, ConnectionConfig};
use dys_observability::middleware::handle_shutdown_signal;

fn topic_from_path(method: HttpMethod, path: impl Into<String> + Display) -> String {
    let new_path = path
        .into()
        .replace("/", ".")
        .replace("{", "by_")
        .replace("}", "");

    let method = format!("{method:?}").to_lowercase();

    format!("api.v1{new_path}.{method}")
}

#[cfg(test)]
mod topic_from_path_tests {
    use utoipa::openapi::HttpMethod;
    use super::topic_from_path;

    #[test]
    fn test_no_params() {
        let method = HttpMethod::Get;
        let path = "/stats/recent/all";

        let actual_topic = topic_from_path(method, path);
        assert_eq!(actual_topic, "api.v1.stats.recent.all.get");
    }

    #[test]
    fn test_with_params() {
        let method = HttpMethod::Get;
        let path = "/stats/recent/{combatant_id}";

        let actual_topic = topic_from_path(method, path);
        assert_eq!(actual_topic, "api.v1.stats.recent.by_combatant_id.get");
    }
}

#[tokio::main]
async fn main() {
    let maybe_open_api_path = std::env::var("OPENAPI_SPEC_PATH");

    let api_spec_path = maybe_open_api_path.unwrap_or(
        String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/openapi.json"))
    );

    let api_spec_str = std::fs::read_to_string(api_spec_path).unwrap();
    let api_spec: utoipa::openapi::OpenApi = serde_json::from_str(&api_spec_str).unwrap();

    let (mut router, api) = utoipa_axum::router::OpenApiRouter::with_openapi(api_spec)
        .split_for_parts();

    let nats_client = make_client(ConnectionConfig::default()).await;

    #[derive(Clone)]
    struct ApiDefinition {
        path: String,
        method: HttpMethod,
        _operation: Operation,
    }

    let mut schema_types = HashMap::new();

    for (path, item) in &api.paths.paths {
        if let Some(get) = item.get.as_ref() {
            let api_definition = ApiDefinition {
                path: path.clone(),
                method: HttpMethod::Get,
                _operation: get.to_owned(),
            };

            for param in get.parameters.as_ref().unwrap() {
                println!("    param: {}", param.name);

                match param.schema.as_ref().unwrap() {
                    RefOr::Ref(t) => {
                        println!("\t{}", t.ref_location);
                    }
                    RefOr::T(t) => {
                        match t {
                            Schema::Object(obj) => {
                                println!("\trequired: {:?}", param.required);
                                println!("\tin: {:?}", param.parameter_in);

                                // ZJ-TODO: verify required params exist
                                schema_types.insert(
                                    param.name.clone(),
                                    obj.schema_type.clone()
                                );
                            }
                            _ => panic!("unhandled type! {t:?}")
                        }
                    }
                }
            }

            let nats_client = nats_client.clone();
            let path_clone = path.clone();
            let schema_types = schema_types.clone();

            router = router.route_service(&path, service_fn(move |request: Request| {
                let api_definition = api_definition.clone();
                let nats_client = nats_client.clone();
                let schema_types = schema_types.clone();

                let mut json_object_map = serde_json::Map::new();

                let actual_path = request.uri().path().to_string();
                let query_arg_start_idx = actual_path.find("?").unwrap_or(actual_path.len());
                let query_arg_end_idx = actual_path.find("&").unwrap_or(actual_path.len());
                let query_arg = &actual_path[query_arg_start_idx..query_arg_end_idx];
                if !query_arg.is_empty() {
                    let name_and_value = query_arg.split("=").collect::<Vec<_>>();
                    let name = name_and_value[0];
                    let value_str = name_and_value[1];

                    let value = match schema_types.get(name).unwrap() {
                        SchemaType::Type(ty) => match ty {
                            Type::String => serde_json::Value::String(value_str.to_string()),
                            Type::Integer => serde_json::Value::Number(serde_json::Number::from(value_str.parse::<i64>().unwrap())),
                            Type::Number => serde_json::Value::Number(serde_json::Number::from_f64(value_str.parse::<f64>().unwrap()).unwrap()),
                            Type::Boolean => serde_json::Value::Bool(value_str.parse::<bool>().unwrap()),
                            _ => panic!("unhandled type! {ty:?}")
                        }
                        _ => panic!("unhandled schema type"),
                    };

                    json_object_map.insert(
                        name.to_string(),
                        value,
                    );

                    // ZJ-TODO: loop over any additional & query args
                }

                let path_no_query_args = actual_path[..query_arg_start_idx].to_string();
                let expected_path = path_clone.clone();

                for (actual, expected) in path_no_query_args.split("/").zip(expected_path.split("/")) {
                    if actual == expected {
                        continue;
                    }

                    let expected = expected
                        .replace("{", "")
                        .replace("}", "")
                        .to_string();

                    json_object_map.insert(
                        expected.clone(),
                        match schema_types.get(&expected).unwrap() {
                            SchemaType::Type(ty) => match ty {
                                Type::String => serde_json::Value::String(actual.to_string()),
                                Type::Integer => serde_json::Value::Number(serde_json::Number::from(actual.parse::<i64>().unwrap())),
                                Type::Number => serde_json::Value::Number(serde_json::Number::from_f64(actual.parse::<f64>().unwrap()).unwrap()),
                                Type::Boolean => serde_json::Value::Bool(actual.parse::<bool>().unwrap()),
                                _ => panic!("unhandled type! {ty:?}")
                            }
                            _ => panic!("unhandled schema type"),
                        }
                    );
                }

                let json_request = json!(json_object_map);

                println!("{:?}", request);
                println!("{}", json_request.to_string());

                async move {
                    let reply_topic = nats_client.new_inbox();
                    let Ok(subscriber) = nats_client.subscribe(reply_topic.clone()).await else {
                        return Ok("failed to subscribe to reply topic".into_response());
                    };

                    let request_bytes = json_request.to_string().into_bytes();

                    // ZJ-TODO: headers
                    let topic = topic_from_path(api_definition.method.clone(), api_definition.path.clone());
                    if let Err(publish_error) = nats_client.publish_with_reply_and_headers(
                        topic,
                        reply_topic,
                        HeaderMap::new(),
                        request_bytes.into(),
                    ).await {
                        panic!("failed to publish to nats client: {}", publish_error);
                    }

                    let mut fused_subscriber = subscriber.fuse();

                    let response = tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(5000)) => {
                            Response::builder()
                                .status(400)
                                .body(axum::body::Body::from(Bytes::from("timed out performing request")))
                                .unwrap()
                        },
                        message = fused_subscriber.select_next_some() => {
                            println!("{}", String::from_utf8(message.payload.to_vec()).unwrap());
                            Response::builder()
                                .status(200)
                                .header("Content-Type", "application/json")
                                .body(axum::body::Body::from(message.payload))
                                .unwrap()
                        },
                    };

                    println!("response received! {response:?}");

                    Ok(response)
                }
            }));
        }
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6050").await.unwrap();
    router = router.merge(
        SwaggerUi::new("/swagger")
            .url("/api/openapi.json", api.clone())
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}
