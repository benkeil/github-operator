use thiserror::Error;

pub mod adapter;
pub mod domain;
pub mod extensions;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("SerializationError")]
    SerializationError(serde_json::Error),

    #[error("KubeError: {0}")]
    KubeError(kube::Error),

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
