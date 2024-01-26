pub mod github_repository_spec;
pub mod repository;

pub trait CompareToSpec<Rhs: ?Sized = Self> {
    fn differ_from_spec(&self, spec: &Rhs) -> bool;
}
