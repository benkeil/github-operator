use differ_from_spec::DifferFromSpec;
use garde::Validate;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::conditions_schema;
use crate::domain::model::immutable_string;

// see https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#events

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "github.platform.benkeil.de",
    version = "v1alpha1",
    kind = "RepositoryPermission",
    namespaced,
    shortname = "permission"
)]
#[kube(status = "RepositoryPermissionStatus")]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPermissionSpec {
    #[garde(skip)]
    #[schemars(schema_with = "immutable_string")]
    pub full_name: String,
    #[garde(skip)]
    #[schemars(schema_with = "immutable_string")]
    pub full_team_name: String,
    #[garde(skip)]
    pub permission: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPermissionStatus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "conditions_schema")]
    pub conditions: Vec<Condition>,
    pub healthy: Option<bool>,
}

impl From<RepositoryPermissionSpec> for RepositoryPermissionResponse {
    fn from(spec: RepositoryPermissionSpec) -> Self {
        Self {
            full_team_name: spec.full_team_name,
            permission: spec.permission,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct RepositoryPermissionResponse {
    pub full_team_name: String,
    pub permission: String,
}
