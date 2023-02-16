use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use prometheus_client::metrics::histogram;

pub struct Metrics {
    // air_quality: AirQualityMetrics,
    pub api_metrics: APIMetrics,
}

// pub struct AirQualityMetrics {
//     pub pm25: Counter,
//     pub pm10: Gauge,
// }
//
// impl AirQualityMetrics {
//     fn new(registry: &mut Registry) -> Self {
//         let registry = registry.sub_registry_with_prefix("air_quality");
//     }
// }

#[derive(Clone)]
pub struct APIMetrics {
    pub errors: Counter<u64>,
    pub latency_seconds: Histogram,
}

impl APIMetrics {
    fn new(registry: &mut Registry) -> Self {
        let registry = registry.sub_registry_with_prefix("api");

        let metrics = APIMetrics {
            errors: Counter::default(),
            latency_seconds: Histogram::new(histogram::exponential_buckets(0.01, 2.0, 10)),
        };

        registry.register("errors", "PJP API Errors",
                          metrics.errors.clone());

        metrics
    }
}

impl Metrics {
    /// Creates the [`Metrics`].
    pub fn new(registry: &mut Registry) -> Self {
        Metrics {
            api_metrics: APIMetrics::new(registry),
        }
    }
}
