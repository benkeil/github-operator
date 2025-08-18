pub mod archive_repository_use_case;
pub mod delete_autolink_reference_use_case;
pub mod delete_permissions_use_case;
pub mod get_repository_use_case;
pub mod model;
pub mod reconcile_autolink_reference_use_case;
pub mod reconcile_permissions_use_case;
pub mod reconcile_repository_use_case;
pub mod service;

use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;

pub fn conditions_schema(_: &mut SchemaGenerator) -> Schema {
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
