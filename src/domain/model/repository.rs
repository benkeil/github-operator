use differ_from_spec::DifferFromSpec;
use std::fmt::Debug;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::model::github_repository_spec::GitHubRepositorySpec;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct Repository {
    pub full_name: String,
    pub repository: Option<RepositoryResponse>,
    pub autolink_references: Option<Vec<AutolinkReference>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
pub struct RepositoryResponse {
    pub security_and_analysis: Option<SecurityAndAnalysisResponse>,
    pub delete_branch_on_merge: Option<bool>,
    pub allow_auto_merge: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub allow_update_branch: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
pub struct SecurityAndAnalysisResponse {
    pub secret_scanning: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_push_protection: Option<SecurityAndAnalysisStatusResponse>,
    pub dependabot_security_updates: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_validity_checks: Option<SecurityAndAnalysisStatusResponse>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
pub struct SecurityAndAnalysisStatusResponse {
    pub status: Status,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Enabled,
    Disabled,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct AutolinkReference {
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct AutolinkReferenceResponse {
    pub id: u64,
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

impl From<GitHubRepositorySpec> for Repository {
    fn from(spec: GitHubRepositorySpec) -> Self {
        Self {
            full_name: spec.full_name,
            repository: spec.repository,
            autolink_references: spec.autolink_references,
        }
    }
}

#[cfg(test)]
mod tests {
    use differ_from_spec::DifferFromSpec;

    #[test]
    fn differ_from_spec() {
        let spec = super::RepositoryResponse {
            security_and_analysis: Some(super::SecurityAndAnalysisResponse {
                secret_scanning: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
                secret_scanning_push_protection: None,
                dependabot_security_updates: None,
                secret_scanning_validity_checks: None,
            }),
            delete_branch_on_merge: Some(true),
            allow_auto_merge: None,
            allow_squash_merge: None,
            allow_merge_commit: None,
            allow_rebase_merge: None,
            allow_update_branch: None,
        };
        let actual = super::RepositoryResponse {
            security_and_analysis: Some(super::SecurityAndAnalysisResponse {
                secret_scanning: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
                secret_scanning_push_protection: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
                dependabot_security_updates: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
                secret_scanning_validity_checks: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
            }),
            delete_branch_on_merge: Some(true),
            allow_auto_merge: Some(true),
            allow_squash_merge: Some(true),
            allow_merge_commit: Some(true),
            allow_rebase_merge: Some(true),
            allow_update_branch: Some(true),
        };
        assert!(!actual.differ_from_spec(&spec));
    }
}
