use garde::Validate;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::conditions_schema;
use crate::domain::model::autolink_reference::{
    AutolinkReferenceRequest, AutolinkReferenceResponse,
};

// see https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#events

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "github.platform.benkeil.de",
    version = "v1alpha1",
    kind = "AutolinkReference",
    namespaced
)]
#[kube(status = "AutolinkReferenceStatus")]
//#[serde(rename_all = "camelCase")]
pub struct AutolinkReferenceSpec {
    #[garde(skip)]
    pub full_name: String,
    #[garde(skip)]
    pub key_prefix: String,
    #[garde(skip)]
    pub url_template: String,
    #[garde(skip)]
    pub is_alphanumeric: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AutolinkReferenceStatus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "conditions_schema")]
    pub conditions: Vec<Condition>,
    pub healthy: Option<bool>,
    pub id: Option<u32>,
}

impl From<AutolinkReferenceSpec> for AutolinkReferenceRequest {
    fn from(spec: AutolinkReferenceSpec) -> Self {
        Self {
            key_prefix: spec.key_prefix,
            url_template: spec.url_template,
            is_alphanumeric: spec.is_alphanumeric,
        }
    }
}

impl From<&AutolinkReferenceResponse> for AutolinkReferenceRequest {
    fn from(spec: &AutolinkReferenceResponse) -> Self {
        Self {
            key_prefix: spec.key_prefix.clone(),
            url_template: spec.url_template.clone(),
            is_alphanumeric: spec.is_alphanumeric,
        }
    }
}
