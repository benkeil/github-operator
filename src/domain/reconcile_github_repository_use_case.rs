use differ_from_spec::DifferFromSpec;
use kube::runtime::events::{Event, EventType, Recorder};

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

        if let Some(spec_autolink_references) = &spec.autolink_references {
            let autolink_references = self
                .github_service
                .get_autolink_references(&spec.full_name)
                .await
                .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
            log::debug!("autolink_references: {:#?}", autolink_references);
            if autolink_references.differ_from_spec(spec_autolink_references) {
                log::info!("autolink references needs to be updated");
                self.github_service
                    .update_autolink_references(&spec.full_name, spec_autolink_references)
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
                recorder
                    .publish(Event {
                        action: "autolink-references-updated".into(),
                        reason: "Reconciling".into(),
                        note: Some("Autolink references updated".into()),
                        type_: EventType::Normal,
                        secondary: None,
                    })
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
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
                repository
            }
            Err(_) => return Err(ReconcileGitHubRepositoryUseCaseError::Error),
        })
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {
    Error,
}
