use crate::domain::model::autolink_reference::AutolinkReferenceResponse;
use crate::domain::model::repository::RepositoryResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod autolink_reference;
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
