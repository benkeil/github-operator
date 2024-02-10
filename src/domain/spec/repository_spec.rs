use garde::Validate;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::conditions_schema;
use crate::domain::model::repository::{
    SecurityAndAnalysisResponse, SecurityAndAnalysisStatusResponse, Status,
};
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
//#[serde(rename_all = "camelCase")]
pub struct RepositorySpec {
    #[garde(skip)]
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
