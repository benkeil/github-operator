use crate::domain::model::repository::Repository;
use async_trait::async_trait;

#[async_trait]
pub trait GitHubService {
    async fn get_repository(&self, owner: &str, name: &str) -> Option<Repository>;
}
