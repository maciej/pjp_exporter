mod metrics;
mod pjp;
mod scraper;
mod service;

use crate::pjp::API;
use crate::scraper::Scraper;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use prometheus_client::registry::Registry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    let metrics = metrics::Metrics::new(&mut registry);

    let pjp_api = API::new(metrics.api_metrics.clone());

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

            let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                service::service(registry.clone(), scraper.clone(), req)
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
