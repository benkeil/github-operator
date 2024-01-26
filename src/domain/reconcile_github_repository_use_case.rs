use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::model::repository::Repository;
use crate::domain::port::github_service::GitHubService;

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
    ) -> Result<Repository, ReconcileGitHubRepositoryUseCaseError> {
        log::info!("reconcile: {}", spec.full_name);
        let (owner, name) = spec.full_name.split_once('/').unwrap();
        let repository = self
            .get_or_create_repository(owner, name, &recorder)
            .await?;
        log::info!("repository: {:#?}", repository);

        Ok(repository)
    }


    async fn get_or_create_repository(
        &self,
        owner: &str,
        name: &str,
        recorder: &Recorder,
    ) -> Result<Repository, ReconcileGitHubRepositoryUseCaseError> {
        let repository = self
            .github_service
            .get_repository(owner, name)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        Ok(match repository {
            None => {
                let repository = self
                    .github_service
                    .create_repository(owner, name)
                    .await
                    .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
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
            Some(repository) => repository,
        })
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {
    Error,
}
