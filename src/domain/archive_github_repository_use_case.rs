use crate::domain::model::github_repository_spec::GitHubRepository;
use crate::domain::service::github_service::GitHubService;
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
        log::info!("archive repository: {}", &github_repository.spec.full_name);
        self.github_service
            .archive_repository(&github_repository.spec.full_name)
            .await
            .map_err(|_| ArchiveGitHubRepositoryUseCaseError::Error)
    }
}

pub enum ArchiveGitHubRepositoryUseCaseError {
    Error,
}
