use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::model::repository::{Repository, RepositoryResponse};
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use serde_json::{Map, Value};

pub mod github_repository_spec;
pub mod repository;

pub trait CompareToSpec<Rhs: ?Sized = Self> {
    fn differ_from_spec(&self, spec: &Rhs) -> bool;
    fn differ_from_spec_2(&self, spec: &GitHubRepositorySpec) -> bool;
}

impl From<RepositoryResponse> for Map<String, Value> {
    fn from(value: RepositoryResponse) -> Self {
        let json = serde_json::to_string(&value).unwrap();
        serde_json::from_str::<Map<String, Value>>(&json).unwrap()
    }
}

impl CompareToSpec for RepositoryResponse {
    fn differ_from_spec(&self, spec: &Self) -> bool {
        differ_from_spec(&spec.clone().into(), &self.clone().into())
    }

    fn differ_from_spec_2(&self, spec: &GitHubRepositorySpec) -> bool {
        let spec: Repository = spec.clone().into();
        differ_from_spec(&spec.repository.clone().into(), &self.clone().into())
    }
}

fn differ_from_spec(spec: &Map<String, Value>, actual: &Map<String, Value>) -> bool {
    spec.iter()
        .fold_while(false, |acc, (key, value)| {
            match value {
                Value::Null => {
                    return Continue(acc);
                }
                Value::Object(spec_object) => {
                    match actual.get(key) {
                        Some(Value::Object(actual_object)) => {
                            let differ = differ_from_spec(spec_object, actual_object);
                            if differ {
                                return Done(true);
                            }
                        }
                        _ => return Done(true),
                    };
                }
                Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                    if actual.get(key) != Some(value) {
                        return Done(true);
                    }
                }
                Value::Array(values) => {
                    if let Some(Value::Array(actual_values)) = actual.get(key) {
                        if values.len() != actual_values.len() {
                            return Done(true);
                        }
                        for (value, actual_value) in values.iter().zip(actual_values.iter()) {
                            if value != actual_value {
                                return Done(true);
                            }
                        }
                    }
                }
            };
            Continue(acc)
        })
        .into_inner()
}
