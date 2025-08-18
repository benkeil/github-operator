use std::fmt::Debug;

use differ_from_spec::DifferFromSpec;
use garde::Validate;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::domain::conditions_schema;
use crate::domain::model::immutable_string;
use crate::domain::model::AutoConfigureSpec;

// see https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#events

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "github.platform.benkeil.de",
    version = "v1alpha1",
    kind = "Repository",
    namespaced,
    shortname = "repo"
)]
#[kube(status = "RepositoryStatus")]
#[serde(rename_all = "camelCase")]
pub struct RepositorySpec {
    #[garde(skip)]
    #[schemars(schema_with = "immutable_string")]
    pub full_name: String,
    #[garde(skip)]
    pub security_and_analysis: Option<SecurityAndAnalysisResponse>,
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
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStatus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "conditions_schema")]
    pub conditions: Vec<Condition>,
    pub healthy: Option<bool>,
}

impl AutoConfigureSpec for RepositorySpec {
    /// Enable additional settings if necessary
    fn auto_configure(&mut self) -> &Self {
        self.security_and_analysis.auto_configure();
        self
    }
}

impl AutoConfigureSpec for Option<SecurityAndAnalysisResponse> {
    fn auto_configure(&mut self) -> &Self {
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
        self
    }
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
