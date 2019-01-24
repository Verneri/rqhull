extern crate bindgen;
extern crate cc;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice::Iter;
use std::env;



fn main() {


    let qhull_dir = Path::new("./qhull");

    update_submodules(&[qhull_dir]);
    build_qhull(qhull_dir);
    build_helper()

}




fn update_submodules(git_submodules: &[&Path]) {

    let iter: Iter<&Path> = git_submodules.iter();
    let mut git_dirs = iter.map(|p| p.join(".git"));
    if git_dirs.any(|p| !p.exists() ) {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
            .status();

    }

}

fn build_helper() {
    cc::Build::new()
        .file("src/helper.c")
        .compile("helper");
}

fn build_qhull(source_dir: &Path) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_src_path = Path::new("gen-src");
    let bindinds_filename = "bindings.rs";
    let bindings_file = gen_src_path.join(bindinds_filename);


    use cmake;
    let dst = cmake::build(source_dir);

    use std::fs;

    #[cfg(feature = "gen-code")]
        {
            let bindings = bindgen::Builder::default().
                header(source_dir.join("src/libqhull_r/qhull_ra.h").to_str().expect("problem with header file path")).
                generate().expect("unable to generate rust bindings for re-entrant qhull");
            if !gen_src_path.exists() {
                fs::create_dir(gen_src_path).expect("unable to generate folder for generated sources");
            } else if !gen_src_path.is_dir() {
                panic!("a non directory file {} exists", gen_src_path.display());
            }

            bindings
                .write_to_file(&bindings_file)
                .expect("Couldn't write bindings!");
        }

    fs::copy(&bindings_file, &out_path.join(bindinds_filename)).expect(&format!("can't copy bindings file {} to {}", &bindings_file.display(), &out_path.display()));



    let libname = if env::var("OPT_LEVEL").map(|ol| ol == "0").unwrap_or(false) {
        "qhullstatic_rd"
    } else {
        "qhullstatic_r"
    };

    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=static={}", libname);

}
