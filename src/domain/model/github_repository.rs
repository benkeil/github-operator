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
    pub actions: Option<ActionsSettings>,
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
    pub security_and_analysis: Option<SecurityAndAnalysis>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
pub struct ActionsSettings {
    pub access: Option<AccessSettings>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum AccessSettings {
    None,
    Organization,
    Enterprise,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct AutolinkReference {
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct GitHubRepositoryStatus {
    // pub security_and_analysis: Option<SecurityAndAnalysis>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecurityAndAnalysis {
    pub secret_scanning: Option<Status>,
    pub secret_scanning_push_protection: Option<Status>,
    pub dependabot_security_updates: Option<Status>,
    pub secret_scanning_validity_checks: Option<Status>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub enum Status {
    Enabled,
    Disabled,
}
