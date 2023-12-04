use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::github_repository::GitHubRepositorySpec;
use crate::domain::model::repository::{Repository, Status};
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
        log::info!("reconcile: {}", spec.slug);
        let (owner, name) = spec.slug.split_once('/').unwrap();
        let repository = self.get_or_create_repository(owner, name, &recorder).await?;
        log::info!("repository: {:#?}", repository);

        if repository.security_and_analysis.secret_scanning != Status::Enabled {
            self.set_secret_scanning(owner, name, &recorder).await?;
        }

        if repository.security_and_analysis.secret_scanning_push_protection != Status::Enabled {
            self.set_secret_scanning_push_protection(owner, name, &recorder).await?
        }

        if repository.security_and_analysis.dependabot_security_updates != Status::Enabled {
            self.set_dependabot_security_updates(owner, name, &recorder).await?
        }

        if repository.security_and_analysis.secret_scanning_validity_checks != Status::Enabled {
            self.set_secret_scanning_validity_checks(owner, name, &recorder).await?
        }

        Ok(repository)
    }

    async fn get_or_create_repository(self, owner: &str, name: &str, recorder: &Recorder) -> Repository {
        let repository = self
            .github_service
            .get_repository(owner, name)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        match repository {
            None => {
                let repository = self.github_service.create_repository(owner, name).await
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
        }
    }

    async fn set_secret_scanning(self, owner: &str, name: &str, recorder: &Recorder) {
        self.github_service
            .set_secret_scanning(owner, name, Status::Enabled)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        recorder
            .publish(Event {
                action: "secret-scanning-enabled".into(),
                reason: "Reconciling".into(),
                note: Some("Enabled secret scanning".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
    }

    async fn set_secret_scanning_push_protection(self, owner: &str, name: &str, recorder: &Recorder) {
        self.github_service
            .set_secret_scanning_push_protection(owner, name, Status::Enabled)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        recorder
            .publish(Event {
                action: "secret-scanning-push-protection-enabled".into(),
                reason: "Reconciling".into(),
                note: Some("Enabled secret scanning push protection".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
    }

    async fn set_dependabot_security_updates(self, owner: &str, name: &str, recorder: &Recorder) {
        self.github_service
            .set_dependabot_security_updates(owner, name, Status::Enabled)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        recorder
            .publish(Event {
                action: "dependabot-security-updates-enabled".into(),
                reason: "Reconciling".into(),
                note: Some("Enabled Dependabot security updates".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
    }

    async fn set_secret_scanning_validity_checks(self, owner: &str, name: &str, recorder: &Recorder) {
        self.github_service
            .set_secret_scanning_validity_checks(owner, name, Status::Enabled)
            .await
            .map_err(|e| ReconcileGitHubRepositoryUseCaseError::Error)?;
        recorder
            .publish(Event {
                action: "secret-scanning-validity-checks-enabled".into(),
                reason: "Reconciling".into(),
                note: Some("Enabled secret scanning validity checks".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(|_| ReconcileGitHubRepositoryUseCaseError::Error)?;
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {
    Error,
}
