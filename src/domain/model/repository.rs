use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::domain::model::github_repository_spec::GitHubRepositorySpec;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct Repository {
    pub repository: RepositoryResponse,
    pub autolink_references: Option<Vec<AutolinkReference>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct RepositoryResponse {
    pub full_name: String,
    pub security_and_analysis: Option<SecurityAndAnalysisResponse>,
    pub delete_branch_on_merge: Option<bool>,
    pub allow_auto_merge: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub allow_update_branch: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct SecurityAndAnalysisResponse {
    pub secret_scanning: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_push_protection: Option<SecurityAndAnalysisStatusResponse>,
    pub dependabot_security_updates: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_validity_checks: Option<SecurityAndAnalysisStatusResponse>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct SecurityAndAnalysisStatusResponse {
    pub status: Status,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Enabled,
    Disabled,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AutolinkReference {
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

impl From<GitHubRepositorySpec> for Repository {
    fn from(spec: GitHubRepositorySpec) -> Self {
        Self {
            repository: RepositoryResponse {
                full_name: spec.full_name,
                security_and_analysis: spec.security_and_analysis,
                delete_branch_on_merge: spec.delete_branch_on_merge,
                allow_auto_merge: spec.allow_auto_merge,
                allow_squash_merge: spec.allow_squash_merge,
                allow_merge_commit: spec.allow_merge_commit,
                allow_rebase_merge: spec.allow_rebase_merge,
                allow_update_branch: spec.allow_update_branch,
            },
            autolink_references: spec.autolink_references,
        }
    }
}