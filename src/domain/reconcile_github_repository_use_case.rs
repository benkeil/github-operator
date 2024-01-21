use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::github_repository::{GitHubRepositorySpec, Status};
use crate::domain::model::repository::Repository;
use crate::domain::model::update_repository::UpdateRepository;
use crate::domain::model::RepositoryChanged;
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

        self.set_security_settings(&repository, spec, &recorder)
            .await?;

        let update_repository = UpdateRepository::from(spec);
        if repository.changed(&update_repository) {
            self.github_service
                .update_repository(owner, name, &update_repository)
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

        Ok(repository)
    }

    async fn set_security_settings(
        &self,
        repository: &Repository,
        spec: &GitHubRepositorySpec,
        recorder: &Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
        let (owner, name) = spec.full_name.split_once('/').unwrap();
        if let Some(security_and_analysis) = &spec.security_and_analysis {
            if let Some(secret_scanning) = &security_and_analysis.secret_scanning {
                if repository.security_and_analysis.secret_scanning != *secret_scanning {
                    self.set_secret_scanning(owner, name, recorder).await?;
                }
            }

            if let Some(secret_scanning_push_protection) =
                &security_and_analysis.secret_scanning_push_protection
            {
                if repository
                    .security_and_analysis
                    .secret_scanning_push_protection
                    != *secret_scanning_push_protection
                {
                    self.set_secret_scanning_push_protection(owner, name, recorder)
                        .await?;
                }
            }

            if let Some(dependabot_security_updates) =
                &security_and_analysis.dependabot_security_updates
            {
                if repository.security_and_analysis.dependabot_security_updates
                    != *dependabot_security_updates
                {
                    self.set_dependabot_security_updates(owner, name, recorder)
                        .await?;
                }
            }

            if let Some(secret_scanning_validity_checks) =
                &security_and_analysis.secret_scanning_validity_checks
            {
                if repository
                    .security_and_analysis
                    .secret_scanning_validity_checks
                    != *secret_scanning_validity_checks
                {
                    self.set_secret_scanning_validity_checks(owner, name, recorder)
                        .await?;
                }
            }
        }

        Ok(())
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

    async fn set_secret_scanning(
        &self,
        owner: &str,
        name: &str,
        recorder: &Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
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
        Ok(())
    }

    async fn set_secret_scanning_push_protection(
        &self,
        owner: &str,
        name: &str,
        recorder: &Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
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
        Ok(())
    }

    async fn set_dependabot_security_updates(
        &self,
        owner: &str,
        name: &str,
        recorder: &Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
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
        Ok(())
    }

    async fn set_secret_scanning_validity_checks(
        &self,
        owner: &str,
        name: &str,
        recorder: &Recorder,
    ) -> Result<(), ReconcileGitHubRepositoryUseCaseError> {
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
        Ok(())
    }
}

pub enum ReconcileGitHubRepositoryUseCaseError {
    Error,
}

impl From<&GitHubRepositorySpec> for UpdateRepository {
    fn from(value: &GitHubRepositorySpec) -> Self {
        Self {
            delete_branch_on_merge: value.delete_branch_on_merge,
            allow_auto_merge: value.allow_auto_merge,
            allow_squash_merge: value.allow_squash_merge,
            allow_merge_commit: value.allow_merge_commit,
            allow_rebase_merge: value.allow_rebase_merge,
            allow_update_branch: value.allow_update_branch,
        }
    }
}
