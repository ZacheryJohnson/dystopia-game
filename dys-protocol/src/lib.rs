#![allow(clippy::all, clippy::pedantic)]

#[cfg(feature = "http")]
#[path = "../generated/http/mod.rs"]
pub mod http;

#[cfg(feature = "nats")]
#[path = "../generated/nats/mod.rs"]
pub mod nats;
