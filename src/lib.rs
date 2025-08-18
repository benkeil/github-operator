use opentelemetry::{global, KeyValue};

use opentelemetry::metrics::Meter;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_prometheus::exporter;
use opentelemetry_sdk::metrics::MeterProviderBuilder;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use prometheus::Registry;
use thiserror::Error;
use tracing::span;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod adapter;
pub mod domain;
pub mod extensions;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("SerializationError")]
    SerializationError(serde_json::Error),

    #[error("KubeError: {0}")]
    KubeError(kube::Error),

    #[error("ConfigurationError")]
    ConfigurationError,

    #[error("HttpError: {0}")]
    HttpError(Box<ureq::Error>),

    #[error("IoError: {0}")]
    IoError(std::io::Error),

    #[error("PrometheusError: {0}")]
    PrometheusError(prometheus::Error),

    // NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
    // so boxing this error to break cycles
    #[error("FinalizerError: {0}")]
    FinalizerError(Box<kube::runtime::finalizer::Error<ControllerError>>),

    #[error("IllegalDocument")]
    IllegalDocument,

    #[error("UseCaseError")]
    UseCaseError,

    #[error("CRD is not queryable; {0}. Is the CRD installed?")]
    CrdNotFound(kube::Error),

    #[error("AlreadyExists")]
    AlreadyExists,

    #[error("NotFound")]
    NotFound,
}

pub fn init_registry() -> Result<Registry, ControllerError> {
    Ok(Registry::new())
}

// see also: https://broch.tech/posts/rust-tracing-opentelemetry/
// see also: https://github.com/tekul/rust-tracing-otlp/
pub fn init_tracing() -> Result<(), ControllerError> {
    let logging_format = std::env::var("APP_LOGGING_FORMAT")
        .unwrap_or("plain".to_string())
        .to_lowercase();

    let (json, plain) = if logging_format == "json" {
        (Some(tracing_subscriber::fmt::layer().json()), None)
    } else {
        (None, Some(tracing_subscriber::fmt::layer()))
    };

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Couldn't create OTLP exporter");

    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();

    let tracer = provider.tracer("readme_example");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(json)
        .with(plain)
        .with(telemetry)
        .init();

    Ok(())
}
