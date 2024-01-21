pub mod github_repository;
pub mod repository;
pub mod update_repository;

pub trait RepositoryChanged<T> {
    fn changed(&self, other: &T) -> bool;
}
