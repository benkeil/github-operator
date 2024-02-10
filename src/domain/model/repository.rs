use std::fmt::Debug;

use differ_from_spec::DifferFromSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::spec::repository_spec::RepositorySpec;

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
    pub advanced_security: Option<SecurityAndAnalysisStatusResponse>,
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

// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#list-repository-teams
// https://docs.github.com/de/rest/collaborators/collaborators?apiVersion=2022-11-28
// https://docs.github.com/en/rest/orgs/security-managers?apiVersion=2022-11-28#list-security-manager-teams

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct TeamPermission {
    pub org: String,
    pub team_slug: String,
    /// The permissions of team members regarding the repository. Must be one of `pull`, `triage`, `push`, `maintain`, `admin` or the name of an existing custom repository role within the organisation.
    pub permission: String,
}

impl From<&RepositorySpec> for RepositoryResponse {
    fn from(spec: &RepositorySpec) -> Self {
        Self {
            security_and_analysis: spec.security_and_analysis.clone(),
            delete_branch_on_merge: spec.delete_branch_on_merge,
            allow_auto_merge: spec.allow_auto_merge,
            allow_squash_merge: spec.allow_squash_merge,
            allow_merge_commit: spec.allow_merge_commit,
            allow_rebase_merge: spec.allow_rebase_merge,
            allow_update_branch: spec.allow_update_branch,
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
                advanced_security: None,
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
                advanced_security: Some(super::SecurityAndAnalysisStatusResponse {
                    status: super::Status::Enabled,
                }),
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
