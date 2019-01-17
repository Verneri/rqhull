use git2::{Repository, Error};


fn main() {
    let repo = clone_qhull("./qhull").expect("failed to clone qhull");
    checkout_version(repo, "v7.3.0").expect("checkout version failed");
    
}

fn clone_qhull(qhull_dir: &str) -> Result<Repository, Error> {
    let url = "https://github.com/qhull/qhull.git";
    Repository::clone(url, qhull_dir)
}

fn checkout_version(repo : Repository, revision: &str) -> Result<(), Error> {
    use git2::build::CheckoutBuilder;

    let mut build = CheckoutBuilder::new();
    let rev = repo.revparse_single(revision)?;

    repo.checkout_tree(&rev,
                       Some(build
                           .force()))
}