use opentelemetry::metrics::Counter;
use opentelemetry::{global, Key};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use tide::{Body, Middleware, Next, Request, Response, StatusCode};

const ROUTE_KEY: Key = Key::from_static_str("http_route");

struct MetricsConfig {
    route: String,
}

struct RequestCountMiddleware {
    route: String,
    exporter: PrometheusExporter,
    request_count: Counter<u64>,
}

impl RequestCountMiddleware {
    fn new(config: MetricsConfig) -> Self {
        let route = config.route;
        let exporter = opentelemetry_prometheus::exporter().init();
        let meter = global::meter("middleware");

        let request_count = meter
            .u64_counter("http_server_requests_count")
            .with_description("total request count")
            .init();

        Self {
            route,
            exporter,
            request_count,
        }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for RequestCountMiddleware {
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> tide::Result {
        if request.url().path() == self.route {
            let encoder = TextEncoder::new();
            let metric_families = self.exporter.registry().gather();
            let mut result = Vec::new();
            encoder.encode(&metric_families, &mut result)?;
            let mut res = Response::new(StatusCode::Ok);
            res.set_content_type(tide::http::mime::PLAIN);
            res.set_body(Body::from_bytes(result));
            Ok(res)
        } else {
            let labels = vec![ROUTE_KEY.string(request.url().path().to_string())];
            self.request_count.add(1, &labels);
            println!("request counted");
            let res = next.run(request).await;
            Ok(res)
        }
    }
}

#[tokio::main]
async fn main() -> tide::Result<()> {
    let config = MetricsConfig {
        route: "/metrics".to_string(),
    };

    let metrics_middleware = RequestCountMiddleware::new(config);

    let mut app = tide::new();
    app.with(metrics_middleware);
    app.at("/hello").get(hello_world);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

async fn hello_world(_req: Request<()>) -> tide::Result {
    Ok("{\"message\": \"hello world\"}".into())
}
