use async_nats::HeaderMap;
use opentelemetry::propagation::{Extractor, Injector};
use tracing::{info_span, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct NatsHeaderExtractor<'a>(pub &'a HeaderMap);
impl<'a> Extractor for NatsHeaderExtractor<'a> {
    /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| Some(value.as_str()))
    }

    /// Collect all the keys from the HeaderMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .iter()
            .map(|(k, _)| k.as_ref())
            .collect::<Vec<_>>()
    }
}

pub struct NatsHeaderInjector<'a>(pub &'a mut HeaderMap);
impl<'a> Injector for NatsHeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key, value);
    }
}

pub fn propagate_otel_context(header_map: &mut HeaderMap) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&Span::current().context(), &mut NatsHeaderInjector(header_map));
    });
}

pub fn create_span_from(message: &async_nats::Message) -> Option<Span> {
    let headers = match &message.headers {
        Some(headers) => headers,
        None => &HeaderMap::default(),
    };

    let context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&NatsHeaderExtractor(&headers))
    });

    let new_span = info_span!("NATS");
    new_span.set_parent(context);
    Some(new_span)
}