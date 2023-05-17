mod metrics;
mod pjp;
mod scraper;

use crate::pjp::Api;
use crate::scraper::Scraper;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::with_prefix("pjp");
    let metrics = metrics::Metrics::new(&mut registry);

    let pjp_api = Api::new(metrics.api_metrics.clone());

    let scraper = Scraper::new(pjp_api.clone(), metrics.air_quality);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening to incoming connections on {:?}...", &addr);

    let registry = Arc::new(registry);
    let scraper = Arc::new(scraper);

    loop {
        let (stream, _) = listener.accept().await?;

        let registry = registry.clone();
        let scraper = scraper.clone();

        tokio::task::spawn(async move {
            let registry = registry.clone();
            let scraper = scraper.clone();

            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    stream,
                    service_fn(move |req: hyper::Request<Incoming>| {
                        service(registry.clone(), scraper.clone(), req)
                    }),
                )
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

pub(crate) async fn service(
    registry: Arc<Registry>,
    scraper: Arc<Scraper>,
    req: hyper::Request<Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, std::io::Error> {
    let _ = scraper.scrape_station(530, "waw-niepodleglosci").await;

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            let mut buf = String::new();

            encode(&mut buf, &registry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| {
                    let body = Bytes::from(buf);
                    hyper::Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(Full::new(body))
                        .unwrap()
                })
        }
        _ => Ok(hyper::Response::new(Full::new(Bytes::from("Hello World!")))),
    }
}
