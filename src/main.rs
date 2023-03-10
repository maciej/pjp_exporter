mod metrics;
mod pjp;
mod scraper;

use crate::pjp::API;
use crate::scraper::Scraper;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use prometheus_client::registry::Registry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

async fn service(
    registry: Arc<Registry>,
    scraper: Arc<Scraper>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<hyper::body::Bytes>>, std::io::Error> {
    let _ = scraper.scrape_station(530).await;

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            let mut buf = String::new();
            let resp = prometheus_client::encoding::text::encode(&mut buf, &registry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| {
                    let body = hyper::body::Bytes::from(buf);
                    hyper::Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(Full::new(body))
                        .unwrap()
                });

            resp
        }
        _ => Ok(hyper::Response::new(Full::new(hyper::body::Bytes::from(
            "Hello World!",
        )))),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    let metrics = metrics::Metrics::new(&mut registry);

    let pjp_api = API::new(metrics.api_metrics.clone());

    let scraper = Scraper::new(pjp_api.clone(), metrics.air_quality);

    let _ = scraper.scrape_station(530).await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;

    let registry = Arc::new(registry);
    let scraper = Arc::new(scraper);

    loop {
        let (stream, _) = listener.accept().await?;

        let registry = registry.clone();
        let scraper = scraper.clone();

        tokio::task::spawn(async move {
            let registry = registry.clone();
            let scraper = scraper.clone();

            let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                service(registry.clone(), scraper.clone(), req)
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
