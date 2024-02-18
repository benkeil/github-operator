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
    kind = "AutolinkReference",
    namespaced
)]
#[kube(status = "AutolinkReferenceStatus")]
//#[serde(rename_all = "camelCase")]
pub struct AutolinkReferenceSpec {
    #[garde(skip)]
    #[schemars(schema_with = "immutable_string")]
    pub full_name: String,
    #[garde(skip)]
    #[schemars(schema_with = "immutable_string")]
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

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct AutolinkReferenceResponse {
    pub id: u32,
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, DifferFromSpec, Eq, Hash)]
pub struct AutolinkReferenceRequest {
    pub key_prefix: String,
    pub url_template: String,
    pub is_alphanumeric: bool,
}

impl From<AutolinkReferenceResponse> for AutolinkReferenceRequest {
    fn from(response: AutolinkReferenceResponse) -> Self {
        Self {
            key_prefix: response.key_prefix,
            url_template: response.url_template,
            is_alphanumeric: response.is_alphanumeric,
        }
    }
}
