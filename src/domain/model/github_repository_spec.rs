use crate::domain::model::repository::{AutolinkReference, Repository, SecurityAndAnalysisResponse};
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
    pub full_name: String,
    #[garde(skip)]
    pub delete_branch_on_merge: Option<bool>,
    #[garde(skip)]
    pub allow_auto_merge: Option<bool>,
    #[garde(skip)]
    pub allow_squash_merge: Option<bool>,
    #[garde(skip)]
    pub allow_merge_commit: Option<bool>,
    #[garde(skip)]
    pub allow_rebase_merge: Option<bool>,
    #[garde(skip)]
    pub allow_update_branch: Option<bool>,
    #[garde(skip)]
    pub autolink_references: Option<Vec<AutolinkReference>>,
    #[garde(skip)]
    pub security_and_analysis: Option<SecurityAndAnalysisResponse>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositoryStatus {
    // pub security_and_analysis: Option<SecurityAndAnalysis>,
}
