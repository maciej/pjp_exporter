use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;

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
    pub errors: Family<APIErrorLabels, Counter<u64>>,
    pub latency_seconds: Family<APILatencyLabels, Histogram>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct APIErrorLabels {
    pub code: u32,
    pub endpoint: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct APILatencyLabels {
    pub endpoint: String,
}

impl APIMetrics {
    fn new(registry: &mut Registry) -> Self {
        let registry = registry.sub_registry_with_prefix("api");

        let metrics = APIMetrics {
            errors: Family::<APIErrorLabels, Counter>::default(),
            latency_seconds: Family::<APILatencyLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(histogram::exponential_buckets(0.01, 2.0, 10))
            }),
        };

        registry.register("errors", "PJP API Errors", metrics.errors.clone());

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
