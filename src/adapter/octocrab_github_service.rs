use crate::domain::model::repository::Repository;
use crate::domain::port::github_service::GitHubService;
use async_trait::async_trait;
use octocrab::Octocrab;
use std::sync::Arc;

pub struct OctocrabGitHubService {
    octocrab: Arc<Octocrab>,
}

impl OctocrabGitHubService {
    pub fn new(octocrab: Arc<Octocrab>) -> Arc<Self> {
        Arc::new(Self { octocrab })
    }
}

#[async_trait]
impl GitHubService for OctocrabGitHubService {
    async fn get_repository(&self, owner: &str, name: &str) -> Option<Repository> {
        let repository = self.octocrab.repos(owner, name).get().await;
        match repository {
            Ok(repository) => Some(Repository {
                owner: repository.owner.unwrap().login,
                name: repository.name,
                full_name: repository.full_name.unwrap(),
                url: repository.git_url.unwrap().to_string(),
            }),
            Err(_) => None,
        }
    }
}
