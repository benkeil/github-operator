use std::fmt::{Debug, Display};

use differ_from_spec::DifferFromSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::model::github_repository_spec::GitHubRepositorySpec;

pub trait AutoConfigureSpec {
    fn auto_configure(&mut self);
}

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

impl AutoConfigureSpec for RepositoryResponse {
    /// Enable additional settings if necessary
    fn auto_configure(&mut self) {
        self.security_and_analysis.auto_configure();
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec)]
pub struct SecurityAndAnalysisResponse {
    pub advanced_security: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_push_protection: Option<SecurityAndAnalysisStatusResponse>,
    pub dependabot_security_updates: Option<SecurityAndAnalysisStatusResponse>,
    pub secret_scanning_validity_checks: Option<SecurityAndAnalysisStatusResponse>,
}

impl AutoConfigureSpec for Option<SecurityAndAnalysisResponse> {
    fn auto_configure(&mut self) {
        self.as_mut().map(|s| {
            if let Some(SecurityAndAnalysisStatusResponse {
                status: Status::Enabled,
            }) = s.secret_scanning
            {
                log::info!("auto enabled advanced_security");
                s.advanced_security = Some(SecurityAndAnalysisStatusResponse {
                    status: Status::Enabled,
                });
            };
            s
        });
    }
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

// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#list-repository-teams
// https://docs.github.com/de/rest/collaborators/collaborators?apiVersion=2022-11-28
// https://docs.github.com/en/rest/orgs/security-managers?apiVersion=2022-11-28#list-security-manager-teams
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct Permissions {
    pub team: Option<Vec<TeamPermission>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct TeamPermission {
    pub org: String,
    pub team_slug: String,
    /// The permissions of team members regarding the repository. Must be one of `pull`, `triage`, `push`, `maintain`, `admin` or the name of an existing custom repository role within the organisation.
    pub permission: String,
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
