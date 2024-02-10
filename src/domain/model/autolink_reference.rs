use differ_from_spec::DifferFromSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
