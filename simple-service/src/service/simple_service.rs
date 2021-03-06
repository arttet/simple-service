use tonic::{Request, Response, Status};

use api::pb::simple_service_server::SimpleService;
use api::pb::{HelloRequest, HelloResponse};

use opentelemetry::global::{get_text_map_propagator, BoxedTracer};
use opentelemetry::trace::mark_span_as_active;
use opentelemetry::trace::Tracer;

#[derive(Debug)]
// Implementation of the server trait generated by tonic
pub struct SimpleServiceServerImpl {
    tracer: BoxedTracer,
}

impl SimpleServiceServerImpl {
    pub fn new() -> Self {
        Self {
            tracer: opentelemetry::global::tracer("SimpleServiceServer"),
        }
    }
}

#[tonic::async_trait]
impl SimpleService for SimpleServiceServerImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let parent_cx =
            get_text_map_propagator(|prop| prop.extract(&MetadataMap(request.metadata())));
        let span = self.tracer.start_with_context("say_hello", &parent_cx);
        let _guard = mark_span_as_active(span);

        let response = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(response))
    }
}

struct MetadataMap<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> opentelemetry::propagation::Extractor for MetadataMap<'a> {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    /// Collect all the keys from the MetadataMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}
