use std::path::PathBuf;

use bundle::Workspace;

pub fn load_testcase(name: &str) -> Workspace {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testcases")
        .join(name);
    Workspace::load(&manifest_dir).expect("invalid testcase")
}
