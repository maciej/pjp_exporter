use crate::scraper::Scraper;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::Response;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::Arc;

pub(crate) async fn service(
    registry: Arc<Registry>,
    scraper: Arc<Scraper>,
    req: hyper::Request<Incoming>,
) -> Result<Response<Full<Bytes>>, std::io::Error> {
    let _ = scraper.scrape_station(530).await;

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            let mut buf = String::new();
            let resp = encode(&mut buf, &registry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| {
                    let body = Bytes::from(buf);
                    Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(Full::new(body))
                        .unwrap()
                });

            resp
        }
        _ => Ok(Response::new(Full::new(Bytes::from("Hello World!")))),
    }
}
