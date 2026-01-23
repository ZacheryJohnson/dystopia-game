use async_nats::HeaderMap;
use axum::extract::Request;
use axum::http::Response;
use axum::response::IntoResponse;
use axum::Router;
use bytes::Bytes;
use dys_nats::connection::{make_client, ConnectionConfig};
use dys_observability::middleware::handle_shutdown_signal;
use futures::StreamExt;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fmt::Display;
use std::time::Duration;
use tower::service_fn;
use utoipa::openapi::path::Operation;
use utoipa::openapi::schema::SchemaType;
use utoipa::openapi::{HttpMethod, OpenApi, RefOr, Schema, Type};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

fn topic_from_path(
    method: HttpMethod,
    path: impl Into<String> + Display,
) -> String {
    let new_path = path
        .into()
        .replace("/", ".")
        .replace("{", "by_")
        .replace("}", "");

    let method = format!("{method:?}").to_lowercase();

    // ZJ-TODO: assuming v1
    format!("api.v1{new_path}.{method}")
}

#[cfg(test)]
mod topic_from_path_tests {
    use super::topic_from_path;
    use utoipa::openapi::HttpMethod;

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

    let api_spec_path = maybe_open_api_path.unwrap_or(String::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/openapi.json"
    )));

    let api_spec_str = std::fs::read_to_string(api_spec_path).unwrap();
    let api_spec: OpenApi = serde_json::from_str(&api_spec_str).unwrap();

    let (mut router, api) = OpenApiRouter::with_openapi(api_spec).split_for_parts();

    let nats_client = make_client(ConnectionConfig::default()).await;

    #[derive(Clone)]
    struct ApiDefinition {
        path: String,
        method: HttpMethod,
    }

    let mut schema_types = HashMap::new();

    let mut register_api_fn = |mut router: Router,
                               path: String,
                               method: HttpMethod,
                               operation: &Operation|
     -> Router {
        let api_definition = ApiDefinition {
            path: path.clone(),
            method: method.clone(),
        };

        if let Some(params) = operation.parameters.as_ref() {
            for param in params {
                match param.schema.as_ref().unwrap() {
                    RefOr::Ref(_) => {
                        unimplemented!(
                            "schema references are currently unimplemented"
                        )
                    }
                    RefOr::T(t) => {
                        match t {
                            Schema::Object(obj) => {
                                // ZJ-TODO: verify required params exist
                                schema_types.insert(
                                    param.name.clone(),
                                    obj.schema_type.clone(),
                                );
                            }
                            _ => panic!("unhandled type! {t:?}"),
                        }
                    }
                }
            }
        }

        let nats_client = nats_client.clone();
        let schema_types = schema_types.clone();

        // We could instead nest here,
        // but ideally /api/... is handled by nginx/middleware
        let api_path = format!(
            "{}{path}",
            std::env::var("API_PREFIX").unwrap_or_default()
        );

        router = router.route_service(&api_path.clone(), service_fn(move |request: Request| {
            let api_definition = api_definition.clone();
            let nats_client = nats_client.clone();
            let schema_types = schema_types.clone();

            let mut json_object_map = serde_json::Map::new();

            let query_arg = request.uri().query().unwrap_or_default();
            if !query_arg.is_empty() {
                let name_and_value = query_arg.split("=").collect::<Vec<_>>();
                let name = name_and_value[0];
                let value_str = name_and_value[1];

                let value = match schema_types.get(name).unwrap() {
                    SchemaType::Type(ty) => match ty {
                        Type::String => Value::String(value_str.to_string()),
                        Type::Integer => Value::Number(serde_json::Number::from(value_str.parse::<i64>().unwrap())),
                        Type::Number => Value::Number(serde_json::Number::from_f64(value_str.parse::<f64>().unwrap()).unwrap()),
                        Type::Boolean => Value::Bool(value_str.parse::<bool>().unwrap()),
                        _ => panic!("unhandled type! {ty:?}")
                    }
                    _ => panic!("unhandled schema type"),
                };

                json_object_map.insert(
                    name.to_string(),
                    value,
                );
            }

            let actual_path = request.uri().path().to_string();
            let expected_path = api_path.clone();

            for (actual, expected) in actual_path.split("/").zip(expected_path.split("/")) {
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
                            Type::String => Value::String(actual.to_string()),
                            Type::Integer => Value::Number(serde_json::Number::from(actual.parse::<i64>().unwrap())),
                            Type::Number => Value::Number(serde_json::Number::from_f64(actual.parse::<f64>().unwrap()).unwrap()),
                            Type::Boolean => Value::Bool(actual.parse::<bool>().unwrap()),
                            _ => panic!("unhandled type! {ty:?}")
                        }
                        _ => panic!("unhandled schema type"),
                    }
                );
            }

            async move {
                const MAXIMUM_PAYLOAD_SIZE_BYTES: usize = 4096;
                let (_, body) = request.into_parts();
                let body_bytes = axum::body::to_bytes(
                    body,
                    MAXIMUM_PAYLOAD_SIZE_BYTES,
                ).await.unwrap();

                let body_json: Map<String, Value> = serde_json::from_slice(body_bytes.as_ref()).unwrap_or_default();
                for (k, v) in body_json {
                    json_object_map.insert(k, v);
                }

                let json_request = json!(json_object_map);

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
                            .status(504)
                            .body(axum::body::Body::from(Bytes::from("timed out performing request")))
                            .unwrap()
                    },
                    message = fused_subscriber.select_next_some() => {
                        if let Some(headers) = message.headers && let Some(err) = headers.get("X-Dys-Error") {
                            let status = if err.as_str() == "MalformedRequest" { 400 } else { 500 };
                            Response::builder()
                                .status(status)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .header("Access-Control-Allow-Headers", "*")
                                .header("Access-Control-Allow-Methods", "*")
                                .body(axum::body::Body::empty())
                                .unwrap()
                        } else {
                            Response::builder()
                                .status(200)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .header("Access-Control-Allow-Headers", "*")
                                .header("Access-Control-Allow-Methods", "*")
                                .body(axum::body::Body::from(message.payload))
                                .unwrap()
                        }
                    },
                };

                Ok(response)
            }
        }));

        router
    };

    for (path, item) in &api.paths.paths {
        if let Some(operation) = item.get.as_ref() {
            router = register_api_fn(
                router,
                path.clone(),
                HttpMethod::Get,
                operation,
            );
        }
        if let Some(operation) = item.put.as_ref() {
            router = register_api_fn(
                router,
                path.clone(),
                HttpMethod::Put,
                operation,
            );
        }
        if let Some(operation) = item.post.as_ref() {
            router = register_api_fn(
                router,
                path.clone(),
                HttpMethod::Post,
                operation,
            );
        }
        if let Some(operation) = item.delete.as_ref() {
            router = register_api_fn(
                router,
                path.clone(),
                HttpMethod::Delete,
                operation,
            );
        }
        if let Some(operation) = item.patch.as_ref() {
            router = register_api_fn(
                router,
                path.clone(),
                HttpMethod::Patch,
                operation,
            );
        }
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6050").await.unwrap();
    router = router.merge(
        SwaggerUi::new("/swagger").url("/api/openapi.json", api.clone()),
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}
