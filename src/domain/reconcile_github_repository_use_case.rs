use kube::runtime::events::{Event, EventType, Recorder};
use std::error::Error;
use thiserror::Error;

use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::model::repository::RepositoryResponse;
use crate::domain::service::github_service::GitHubService;

pub struct ReconcileGitHubRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcileGitHubRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        spec: &GitHubRepositorySpec,
        recorder: Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
        log::info!("reconcile: {}", &spec.full_name);

        if let Some(spec_repository) = &spec.repository {
            let repository = self
                .get_or_create_repository(&spec.full_name, &recorder)
                .await?;
            log::debug!("repository: {:#?}", repository);
            if repository.differ_from_spec(spec_repository) {
                log::info!("repository needs to be updated");
                self.github_service
                    .update_repository(&spec.full_name, spec_repository)
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
                recorder
                    .publish(Event {
                        action: "repository-updated".into(),
                        reason: "Reconciling".into(),
                        note: Some("GitHub repository updated".into()),
                        type_: EventType::Normal,
                        secondary: None,
                    })
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
            }
        }

        let autolink_references = self
            .github_service
            .get_autolink_references(&spec.full_name)
            .await
            .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
        log::debug!("autolink_references: {:#?}", autolink_references);

        // TODO the key_prefix IS a unique key
        // TODO the API should throw an error when I try to add a duplicate
        // add references that are actually not present
        let spec_autolink_references = match &spec.autolink_references {
            Some(spec_autolink_references) => spec_autolink_references,
            None => Some(vec![]),
        };
        for spec_autolink_reference in spec_autolink_references {
            if !autolink_references.iter().any(|autolink_reference| {
                spec_autolink_reference.key_prefix == autolink_reference.key_prefix
            }) {
                log::info!("add autolink reference: {:#?}", spec_autolink_reference);
                self.github_service
                    .add_autolink_references(&spec.full_name, spec_autolink_reference)
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
            }
        }

        // delete references that are no longer in the spec
        for ref autolink_reference in autolink_references {
            if !spec_autolink_references
                .iter()
                .any(|spec_autolink_reference| {
                    spec_autolink_reference.key_prefix == autolink_reference.key_prefix
                })
            {
                log::info!("delete autolink reference: {:#?}", autolink_reference);
                self.github_service
                    .delete_autolink_references(&spec.full_name, autolink_reference.id)
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
            }
        }

        Ok(())
    }

    async fn get_or_create_repository(
        &self,
        full_name: &str,
        recorder: &Recorder,
    ) -> Result<RepositoryResponse, ReconcileGitHubRepositoryUseCaseError> {
        let repository = self.github_service.get_repository(full_name).await;
        Ok(match repository {
            Ok(Some(repository)) => repository,
            Ok(None) => {
                let repository = self
                    .github_service
                    .create_repository(full_name)
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
                recorder
                    .publish(Event {
                        action: "repository-created".into(),
                        reason: "Reconciling".into(),
                        note: Some("GitHub repository created".into()),
                        type_: EventType::Normal,
                        secondary: None,
                    })
                    .await
                    .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
                repository
            }
            Err(e) => return Err(ReconcileGitHubRepositoryUseCaseError::Error(e)),
        })
    }
}

#[derive(Error, Debug)]
pub enum ReconcileGitHubRepositoryUseCaseError {
    #[error("ReconcileGitHubRepositoryUseCaseError: {0}")]
    Error(dyn std::error::Error),
}
