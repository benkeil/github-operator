use garde::Validate;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "platform.benkeil.de",
    version = "v1alpha1",
    kind = "GitHubRepository",
    namespaced,
    shortname = "repo"
)]
#[kube(status = "GitHubRepositoryStatus")]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositorySpec {
    #[garde(skip)]
    pub slug: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct GitHubRepositoryStatus {
    // pub security_and_analysis: Option<SecurityAndAnalysis>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecurityAndAnalysis {
    pub secret_scanning: Status,
    pub secret_scanning_push_protection: Status,
    pub dependabot_security_updates: Status,
    pub secret_scanning_validity_checks: Status,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum Status {
    Enabled,
    Disabled,
}
