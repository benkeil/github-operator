use crate::domain::model::github_repository::SecurityAndAnalysis;
use crate::domain::model::repository::Repository;
use crate::domain::model::RepositoryChanged;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use serde::Serialize;

#[derive(PartialEq, Debug, Serialize)]
pub struct UpdateRepository {
    pub delete_branch_on_merge: Option<bool>,
    pub allow_auto_merge: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub allow_update_branch: Option<bool>,
    pub security_and_analysis: Option<SecurityAndAnalysis>,
}

impl UpdateRepository {
    pub fn any_set(&self) -> bool {
        [
            self.delete_branch_on_merge,
            self.allow_auto_merge,
            self.allow_squash_merge,
            self.allow_merge_commit,
            self.allow_rebase_merge,
            self.allow_update_branch,
        ]
        .iter()
        .any(|x| x.is_some())
    }
}

impl RepositoryChanged<UpdateRepository> for Repository {
    fn changed(&self, other: &UpdateRepository) -> bool {
        let security_and_analysis_changed = match &other.security_and_analysis {
            Some(security_and_analysis) => vec![
                (
                    &self.security_and_analysis.dependabot_security_updates,
                    &security_and_analysis.dependabot_security_updates,
                ),
                (
                    &self.security_and_analysis.secret_scanning,
                    &security_and_analysis.secret_scanning,
                ),
                (
                    &self.security_and_analysis.secret_scanning_push_protection,
                    &security_and_analysis.secret_scanning_push_protection,
                ),
                (
                    &self.security_and_analysis.secret_scanning_validity_checks,
                    &security_and_analysis.secret_scanning_validity_checks,
                ),
            ],
            None => vec![],
        }
        .iter()
        .fold_while(false, |acc, (s, o)| {
            if let Some(value) = o {
                if **s != *value {
                    return Done(true);
                }
            }
            Continue(false)
        })
        .into_inner();
        let fields_changed = [
            (self.delete_branch_on_merge, other.delete_branch_on_merge),
            (self.allow_auto_merge, other.allow_auto_merge),
            (self.allow_squash_merge, other.allow_squash_merge),
            (self.allow_merge_commit, other.allow_merge_commit),
            (self.allow_rebase_merge, other.allow_rebase_merge),
            (self.allow_update_branch, other.allow_update_branch),
        ]
        .iter()
        .fold_while(false, |acc, (s, o)| {
            if let Some(value) = o {
                if *s != *value {
                    return Done(true);
                }
            }
            Continue(false)
        })
        .into_inner();
        security_and_analysis_changed && fields_changed
    }
}
