use std::{env, process::Command};

fn get_git_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").expect("no `CARGO_PKG_VERSION`");

    let child = Command::new("git").args(["describe", "--always"]).output();
    match child {
        Ok(child) => {
            let buf = std::str::from_utf8(&child.stdout).expect("stdout not read");
            format!("v{version}\ngit ref: {buf}")
        }
        Err(err) => {
            eprintln!("`git describe` err: {err}");
            version
        }
    }
}

fn main() {
    let version = get_git_version();
    println!("cargo:rustc-env=RUSTDX_VERSION={version}");
}
