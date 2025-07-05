pub mod error;
pub mod connection;

/// The `rpc` module handles bidirectional communication between services.
/// One service makes a request (the "client") and the other service provides a response (the "server").
/// The client should not assume requests will always be handled by the same server.
/// The server can assume that responses will be returned to the original requesting client.
pub mod rpc;

/// The `event` module handles unidirectional communication from a service.
/// One service emits an event that may be handled by any number of services interested in the event.
/// The emitting service should not assume that at least one service will process the event.
/// The receiving service can assume that all events emitted will be delivered (eg fanout).
pub mod event;

mod otel;