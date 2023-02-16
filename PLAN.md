# Project plan


1. Create an API client for PJP along with JSON response data mapping.
2. Figure out how to do integration testing against the real API using GitHub Actions. 
3. Single-sensor exporter.
4. Expand to all sensors.
5. All auxiliary metrics.


## Auxiliary metrics

1. API requests, http_request_count.
2. API latency, http_request_latency_seconds a summary.
3. API errors, http_request_error.


## Metrics library and examples

As the metrics library I picked the official prometheus Rust client, `prometheus-client` crate.
Its reverse dependencies can be found [here](https://crates.io/crates/prometheus-client/reverse_dependencies). Out of which I picked
[rust-libp2p](https://github.com/libp2p/rust-libp2p) and [fuel-core](https://github.com/FuelLabs/fuel-core) as exemplars.
