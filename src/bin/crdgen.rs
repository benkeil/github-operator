use github_operator::domain::model::github_repository::GitHubRepository;
use kube::CustomResourceExt;

fn main() {
    let crds = vec![serde_yaml::to_string(&GitHubRepository::crd()).unwrap()];
    print!("{}", crds.join("\n---\n"))
}
