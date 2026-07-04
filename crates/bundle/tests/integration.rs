mod common;
use common::load_testcase;

#[test]
fn bundle_shared_dependency_once() {
    let workspace = load_testcase("shared_dependency");
    let expanded = workspace
        .bundle("fn main() { use namespace::entry; }", "namespace")
        .expect("invalid testcase");

    assert_eq!(expanded.matches("pub mod namespace").count(), 1);
    assert_eq!(expanded.matches("pub mod entry").count(), 1);
    assert_eq!(expanded.matches("pub mod child").count(), 1);
    assert_eq!(expanded.matches("pub mod grandchild").count(), 1);
}

#[test]
fn expand_module() {
    let workspace = load_testcase("expand_module");
    let expanded = workspace
        .expand(&["entry"], "namespace")
        .expect("invalid testcase");

    let actual = prettyplease::unparse(
        &syn::parse_file(
            r#"
            pub mod namespace {
                pub mod entry {
                    mod foo {
                        mod bar {}
                    }
                }
            }"#,
        )
        .expect("invalid testcase"),
    );
    assert_eq!(expanded, actual);
}

#[test]
fn bundle_nothing() {
    let workspace = load_testcase("shared_dependency");

    let rendered = workspace
        .bundle("fn main() {}", "namespace")
        .expect("invalid testcase");

    assert!(rendered.is_empty());
}

#[test]
fn reject_unknown_crate() {
    let workspace = load_testcase("shared_dependency");
    let error = workspace
        .expand(&["unknown"], "namespace")
        .expect_err("invalid testcase");
    assert_eq!(error.to_string(), "unknown crate `unknown`");
}

#[test]
fn reject_crate_reference_path() {
    let workspace = load_testcase("crate_absolute_path");
    let error = workspace
        .expand(&["entry"], "namespace")
        .expect_err("crate absolute path should be rejected");
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testcases")
        .join("crate_absolute_path")
        .join("entry")
        .join("src")
        .join("lib.rs")
        .canonicalize()
        .expect("invalid testcase");
    assert_eq!(
        error.to_string(),
        format!(
            "crate source `{}` contains unsupported `crate` reference; fix the library",
            source_path.display()
        )
    );
}
