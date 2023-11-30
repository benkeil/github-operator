use crate::domain::model::github_repository::GitHubRepository;
use crate::domain::model::repository::{Repository, Status};
use crate::domain::port::github_service::GitHubService;
use kube::runtime::events::{Event, EventType, Recorder};
use kube::Client;
use serde_json::json;
use std::sync::Arc;

pub struct ReconcileGitHubRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcileGitHubRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        github_repository: Arc<GitHubRepository>,
        recorder: Recorder,
    ) -> Result<Repository, ReconcileGitHubRepositoryUseCaseError> {
        log::info!("reconcile: {}", github_repository.spec.slug);
        let (owner, name) = github_repository.spec.slug.split_once('/').unwrap();
        let repository = self
            .github_service
            .get_repository(owner, name)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        let repository = match repository {
            Some(repository) => {
                log::info!("repository found: {:#?}", repository);
                self.github_service
                    .set_secret_scanning(owner, name, Status::Enabled)
                    .await
                    .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;

                recorder
                    .publish(Event {
                        action: "Update GitHub repository".into(),
                        reason: "Reconciling".into(),
                        note: Some("updated security settings".into()),
                        type_: EventType::Normal,
                        secondary: None,
                    })
                    .await
                    .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;

                repository
            }
            None => self
                .github_service
                .create_repository(owner, name)
                .await
                .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?,
        };
        Ok(repository)
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {
    Error,
}
