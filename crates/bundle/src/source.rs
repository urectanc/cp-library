use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use syn::visit::Visit;

pub(crate) fn names_under_namespace(source: &str, namespace: &str) -> Result<Vec<String>> {
    let file = syn::parse_file(source).context("parsing bundle input")?;

    let mut collector = Collector::new(namespace);
    collector.visit_file(&file);
    collector.finish()
}

struct Collector<'a> {
    namespace: &'a str,
    names: BTreeSet<String>,
    glob_found: bool,
}

impl<'a> Collector<'a> {
    fn new(namespace: &'a str) -> Self {
        Self {
            namespace,
            names: BTreeSet::new(),
            glob_found: false,
        }
    }

    fn finish(self) -> Result<Vec<String>> {
        ensure!(
            !self.glob_found,
            "glob import `{}::*` not supported",
            self.namespace
        );

        Ok(self.names.into_iter().collect())
    }

    fn visit_namespace_use_tree(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => {
                self.names.insert(path.ident.to_string());
            }
            syn::UseTree::Name(name) => {
                self.names.insert(name.ident.to_string());
            }
            syn::UseTree::Rename(rename) => {
                self.names.insert(rename.ident.to_string());
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.visit_namespace_use_tree(item);
                }
            }
            syn::UseTree::Glob(_) => {
                self.glob_found = true;
            }
        }
    }
}

impl Visit<'_> for Collector<'_> {
    fn visit_item_use(&mut self, item_use: &syn::ItemUse) {
        match &item_use.tree {
            syn::UseTree::Path(path) if path.ident == self.namespace => {
                self.visit_namespace_use_tree(&path.tree);
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    if let syn::UseTree::Path(path) = item
                        && path.ident == self.namespace
                    {
                        self.visit_namespace_use_tree(&path.tree);
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_path(&mut self, path: &syn::Path) {
        if let (Some(head), Some(crate_segment)) = (path.segments.get(0), path.segments.get(1))
            && head.ident == self.namespace
        {
            self.names.insert(crate_segment.ident.to_string());
        }
        syn::visit::visit_path(self, path);
    }
}

#[cfg(test)]
mod tests {
    use super::names_under_namespace;

    fn test(source: &str, expected: Vec<String>) {
        let names = names_under_namespace(source, "namespace").expect("invalid testcase");
        assert_eq!(names, expected);
    }

    #[test]
    fn collect_use() {
        test("use namespace::bundled::*;", vec![String::from("bundled")]);
    }

    #[test]
    fn collect_path() {
        test(
            "fn f() { namespace::bundled::C }",
            vec![String::from("bundled")],
        );
    }

    #[test]
    fn collect_nested_path() {
        test(
            "type V = Vec<namespace::bundled::T>;",
            vec![String::from("bundled")],
        );
    }

    #[test]
    fn ignore_non_namespace() {
        test("use std::collections::*;", vec![]);
    }

    #[test]
    fn reject_glob() {
        names_under_namespace("use namespace::*;", "namespace").expect_err("glob should fail");
    }
}
