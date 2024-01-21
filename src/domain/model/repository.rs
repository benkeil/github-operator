use crate::domain::model::github_repository::{ActionsSettings, AutolinkReference, Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub security_and_analysis: SecurityAndAnalysis,
    pub autolink_references: Vec<AutolinkReference>,
    // pub actions: ActionsSettings,
    pub delete_branch_on_merge: bool,
    pub allow_auto_merge: bool,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase_merge: bool,
    pub allow_update_branch: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityAndAnalysis {
    pub secret_scanning: Status,
    pub secret_scanning_push_protection: Status,
    pub dependabot_security_updates: Status,
    pub secret_scanning_validity_checks: Status,
}
