use crate::domain::model::autolink_reference::AutolinkReferenceResponse;
use crate::domain::model::repository::RepositoryResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod autolink_reference;
pub mod permission;
pub mod repository;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct RepositoryFullView {
    pub full_name: String,
    pub repository: Option<RepositoryResponse>,
    pub autolink_references: Option<Vec<AutolinkReferenceResponse>>,
}

pub trait AutoConfigureSpec {
    fn auto_configure(&mut self) -> &Self;
}

// https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definitions/#validation-rules
pub fn immutable_string(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "string",
        "x-kubernetes-validations": [{
            "rule": "self == oldSelf",
            "message": "Value is immutable"
        }]
    }))
    .unwrap()
}
