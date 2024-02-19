use thiserror::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
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

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Couldn't create OTLP tracer");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(json)
        .with(plain)
        .with(telemetry)
        .init();

    Ok(())
}
