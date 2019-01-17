use git2::{Repository, Error};


fn main() {
    clone_qhull("./qhull");
}

fn clone_qhull(qhull_dir: &str) -> Result<Repository, Error> {
    let url = "https://github.com/qhull/qhull.git";
    Repository::clone(url, qhull_dir)
}