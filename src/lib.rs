use thiserror::Error;

pub mod domain;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("GitHubError: {0}")]
    GitHubError(octocrab::Error),

    #[error("SerializationError")]
    SerializationError(serde_json::Error),

    #[error("KubeError: {0}")]
    KubeError(kube::Error),

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
}

// impl std::fmt::Display for ControllerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         // TODO use anyhow
//         match self {
//             ControllerError::FinalizerError(_) => f.write_str("FinalizerError"),
//             ControllerError::KubeError(_) => f.write_str("KubeError"),
//             ControllerError::SerializationError(_) => f.write_str("SerializationError"),
//             ControllerError::IllegalDocument => f.write_str("IllegalDocument"),
//             ControllerError::CrdNotFound(e) => {
//                 f.write_str(format!("CRD is not queryable; {e:?}. Is the CRD installed?").as_str())
//             }
//         }
//     }
// }

// impl std::error::Error for ControllerError {}
