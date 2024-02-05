use garde::Validate;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::model::repository::{AutolinkReference, Permissions, RepositoryResponse};

// see https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#events

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(
    group = "platform.benkeil.de",
    version = "v1alpha1",
    kind = "GitHubRepository",
    namespaced,
    shortname = "repo"
)]
#[kube(status = "GitHubRepositoryStatus")]
//#[serde(rename_all = "camelCase")]
pub struct GitHubRepositorySpec {
    #[garde(skip)]
    pub full_name: String,
    #[garde(skip)]
    pub repository: Option<RepositoryResponse>,
    #[garde(skip)]
    pub autolink_references: Option<Vec<AutolinkReference>>,
    #[garde(skip)]
    pub permissions: Option<Permissions>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositoryStatus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "conditions")]
    pub conditions: Vec<Condition>,
    pub healthy: Option<bool>,
}

struct ConditionWrapper(Condition);

impl schemars::JsonSchema for ConditionWrapper {
    fn schema_name() -> String {
        "conditions".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        todo!()
    }
}

fn conditions(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "array",
        "x-kubernetes-list-type": "map",
        "x-kubernetes-list-map-keys": ["type"],
        "items": {
            "type": "object",
            "properties": {
                "lastTransitionTime": { "format": "date-time", "type": "string" },
                "message": { "type": "string" },
                "observedGeneration": { "type": "integer", "format": "int64", "default": 0 },
                "reason": { "type": "string" },
                "status": { "type": "string" },
                "type": { "type": "string" }
            },
            "required": [
                "lastTransitionTime",
                "message",
                "reason",
                "status",
                "type"
            ],
        },
    }))
    .unwrap()
}
