use crate::domain::model::github_repository_spec::GitHubRepository;
use crate::domain::port::github_service::GitHubService;
use std::sync::Arc;

pub struct ArchiveGitHubRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ArchiveGitHubRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        github_repository: Arc<GitHubRepository>,
    ) -> Result<(), ArchiveGitHubRepositoryUseCaseError> {
        let (owner, name) = github_repository.spec.full_name.split_once('/').unwrap();
        log::info!("archive repository: {}/{}", owner, name);
        self.github_service
            .archive_repository(owner, name)
            .await
            .map_err(|_| ArchiveGitHubRepositoryUseCaseError::Error)
    }
}

pub enum ArchiveGitHubRepositoryUseCaseError {
    Error,
}
