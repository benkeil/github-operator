use std::fmt::Debug;

use differ_from_spec::DifferFromSpec;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::model::CompareToSpec;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct Repository {
    pub repository: RepositoryResponse,
    pub autolink_references: Option<Vec<AutolinkReference>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
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
//#[serde(rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use crate::domain::model::CompareToSpec;

    #[test]
    fn differ_from_spec() {
        let spec = super::RepositoryResponse {
            full_name: "foo/bar".into(),
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
            full_name: "foo/bar".into(),
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
        assert_eq!(false, actual.differ_from_spec(&spec));
    }

    #[test]
    fn derive() {
        let spec = super::RepositoryResponse {
            full_name: "foo/bar".into(),
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
        super::RepositoryResponse::describe();
    }
}
