use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::service::github_service::GitHubService;

pub struct ArchiveGitHubRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ArchiveGitHubRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        spec: &GitHubRepositorySpec,
    ) -> Result<(), ArchiveGitHubRepositoryUseCaseError> {
        log::info!("archive repository: {}", &spec.full_name);
        self.github_service
            .archive_repository(&spec.full_name)
            .await
            .map_err(|_| ArchiveGitHubRepositoryUseCaseError::Error)
    }
}

pub enum ArchiveGitHubRepositoryUseCaseError {
    Error,
}
