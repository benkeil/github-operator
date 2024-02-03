use garde::Validate;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::model::repository::{AutolinkReference, Permissions, RepositoryResponse};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "platform.benkeil.de",
    version = "v1alpha1",
    kind = "GitHubRepository",
    namespaced,
    shortname = "repo"
)]
#[kube(status = "GitHubRepositoryStatus")]
//#[serde(rename_all = "camelCase")]
pub struct GitHubRepositorySpec {
    #[garde(skip)]
    pub full_name: String,
    #[garde(skip)]
    pub repository: Option<RepositoryResponse>,
    #[garde(skip)]
    pub autolink_references: Option<Vec<AutolinkReference>>,
    #[garde(skip)]
    pub permissions: Option<Permissions>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositoryStatus {
    // pub security_and_analysis: Option<SecurityAndAnalysis>,
}
