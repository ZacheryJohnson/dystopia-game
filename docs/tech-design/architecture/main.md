# Overview

Dystopia is built as a collection of domain services that are accessible through an API gateway. This is not for any deployment constraints or strong business-case rationale: I just wanted to learn how to build an API gateway and have a reasonable deployment methodology.

Service endpoints generate OpenAPI v3 specs via the Rust crate `utoipa`. These specs are currently collected at build time, meaning all services must be deployed together. Ideally, the specs are provided to the gateway at runtime.

The API gateway is a standard web server communicating via HTTP/2. Upon receiving and validating requests from clients, the API gateway uses NATS to queue messages to available services. 
