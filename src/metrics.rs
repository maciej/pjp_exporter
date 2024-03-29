use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;

pub struct Metrics {
    pub air_quality: AirQualityMetrics,
    pub api_metrics: ApiMetrics,
}

pub struct AirQualityMetrics {
    pub pm25: Family<AirQualityLabels, Gauge<f64, AtomicU64>>,
    pub pm10: Family<AirQualityLabels, Gauge<f64, AtomicU64>>,
}

impl AirQualityMetrics {
    fn new(registry: &mut Registry) -> Self {
        let registry = registry.sub_registry_with_prefix("air_quality");

        let metrics = AirQualityMetrics {
            pm10: Family::<AirQualityLabels, Gauge<f64, AtomicU64>>::default(),
            pm25: Family::<AirQualityLabels, Gauge<f64, AtomicU64>>::default(),
        };

        registry.register("pm10", "PM10 pollution", metrics.pm10.clone());
        registry.register("pm25", "PM2.5 pollution", metrics.pm25.clone());

        metrics
    }
}

#[derive(Clone)]
pub struct ApiMetrics {
    pub errors: Family<ApiErrorLabels, Counter<u64>>,
    pub latency_seconds: Family<ApiLabels, Histogram>,
    pub requests: Family<ApiLabels, Counter<u64>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ApiErrorLabels {
    pub code: u16,
    pub endpoint: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct AirQualityLabels {
    pub station: u32,
    pub station_name: String,
    pub sensor: u32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ApiLabels {
    pub endpoint: String,
}

impl ApiMetrics {
    fn new(registry: &mut Registry) -> Self {
        let registry = registry.sub_registry_with_prefix("api");

        let metrics = ApiMetrics {
            errors: Family::<ApiErrorLabels, Counter>::default(),
            latency_seconds: Family::<ApiLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(histogram::exponential_buckets(0.01, 2.0, 10))
            }),
            requests: Family::<ApiLabels, Counter>::default(),
        };

        registry.register("errors", "PJP API Errors", metrics.errors.clone());
        registry.register(
            "latency_seconds",
            "PJP Latency",
            metrics.latency_seconds.clone(),
        );

        metrics
    }
}

impl Metrics {
    /// Creates the [`Metrics`].
    pub fn new(registry: &mut Registry) -> Self {
        Metrics {
            air_quality: AirQualityMetrics::new(registry),
            api_metrics: ApiMetrics::new(registry),
        }
    }
}
