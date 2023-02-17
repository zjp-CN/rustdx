use std::{env::var, fs::File, io::Write, path::Path, process::Command};

fn get_git_version() -> String {
    let version = var("CARGO_PKG_VERSION").expect("no `CARGO_PKG_VERSION`");

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
    let p = Path::new(&var("OUT_DIR").expect("no `OUT_DIR`")).join("VERSION");
    let mut f = File::create(p).unwrap_or_else(|_| panic!("{}", "{p:?} not created"));
    f.write_all(version.trim().as_bytes()).unwrap_or_else(|_| panic!("{}", "{p:?} not written"));
}
