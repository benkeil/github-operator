use crate::domain::model::github_repository::GitHubRepository;
use crate::domain::port::github_service::GitHubService;
use std::sync::Arc;

pub struct ReconcileGitHubRepositoryUseCase {
    github_service: Arc<dyn GitHubService + Send + Sync>,
}

impl ReconcileGitHubRepositoryUseCase {
    pub fn new(github_service: Arc<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        github_repository: Arc<GitHubRepository>,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
        let (owner, name) = github_repository.spec.slug.split_once('/').unwrap();
        let repository = self.github_service.get_repository(owner, name).await;
        if let Some(repository) = repository {
            log::info!("repository found: {:#?}", repository);
        };
        Ok(())
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {}
