use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use syn::{parse_quote, visit::Visit};

use crate::workspace::Crate;

pub(crate) fn build(crates: Vec<&Crate>, namespace: &str) -> Result<syn::File> {
    let crate_modules = crates
        .iter()
        .map(|krate| {
            let loader = Loader::new(krate.src_dir());
            let builder = AstBuilder::new(namespace, krate.dependencies(), loader);
            let crate_module = public_module(krate.name(), builder.build()?)?;
            Ok(syn::Item::Mod(crate_module))
        })
        .collect::<Result<Vec<_>>>()?;
    let namespace_module = public_module(namespace, crate_modules)?;

    Ok(syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: vec![syn::Item::Mod(namespace_module)],
    })
}

struct Loader {
    src_dir: PathBuf,
}

impl Loader {
    fn new(src_dir: PathBuf) -> Self {
        Self { src_dir }
    }

    fn load(&self, module_path: &[String]) -> Result<syn::File> {
        let file_path = self.find(module_path)?;
        let source = std::fs::read_to_string(&file_path)
            .with_context(|| format!("reading crate source `{}`", file_path.display()))?;
        syn::parse_file(&source)
            .with_context(|| format!("parsing crate source `{}`", file_path.display()))
    }

    fn find(&self, module_path: &[String]) -> Result<PathBuf> {
        let Some((module_name, parent_path)) = module_path.split_last() else {
            return Ok(self.src_dir.join("lib.rs"));
        };

        let mut module_dir = self.src_dir.clone();
        module_dir.extend(parent_path);

        let name_rs = module_dir.join(format!("{module_name}.rs"));
        if name_rs.is_file() {
            return Ok(name_rs);
        }

        let name_mod_rs = module_dir.join(module_name).join("mod.rs");
        if name_mod_rs.is_file() {
            return Ok(name_mod_rs);
        }

        bail!(
            "module source file not found for `{}`",
            module_path.join("::")
        )
    }
}

struct AstBuilder<'a> {
    namespace: &'a str,
    dependencies: &'a BTreeSet<String>,
    loader: Loader,
}

impl<'a> AstBuilder<'a> {
    fn new(namespace: &'a str, dependencies: &'a BTreeSet<String>, loader: Loader) -> Self {
        Self {
            namespace,
            dependencies,
            loader,
        }
    }

    fn build(&self) -> Result<Vec<syn::Item>> {
        let root = self.loader.load(&[])?;
        self.expand_module(root.items, &[])
    }

    fn expand_module(
        &self,
        mut ast: Vec<syn::Item>,
        module_path: &[String],
    ) -> Result<Vec<syn::Item>> {
        let module_dependencies = self.module_dependencies(&ast);

        for item in &mut ast {
            let syn::Item::Mod(item_mod) = item else {
                continue;
            };
            let mut child_path = module_path.to_owned();
            child_path.push(item_mod.ident.to_string());
            let child_items = match item_mod.content.take() {
                // mod foo { ... }
                Some((_, items)) => items,
                // mod foo;
                None => self.loader.load(&child_path)?.items,
            };
            let child_ast = self.expand_module(child_items, &child_path)?;
            item_mod.content = Some((syn::token::Brace::default(), child_ast));
            item_mod.semi = None;
        }

        if !module_dependencies.is_empty() {
            let namespace = parse_ident(self.namespace)?;
            let dependencies = module_dependencies
                .iter()
                .map(|dependency| parse_ident(dependency))
                .collect::<Result<Vec<_>, _>>()?;

            let dependency_import = if dependencies.len() == 1 {
                let dependency = &dependencies[0];
                parse_quote!(use crate::#namespace::#dependency;)
            } else {
                parse_quote!(use crate::#namespace::{#(#dependencies),*};)
            };
            ast.insert(0, dependency_import);
        }
        Ok(ast)
    }

    fn module_dependencies(&self, ast: &[syn::Item]) -> BTreeSet<String> {
        let mut scanner = DependencyScanner::new(self.dependencies);
        for item in ast {
            scanner.visit_item(item);
        }
        scanner.finish()
    }
}

struct DependencyScanner<'a> {
    crate_dependencies: &'a BTreeSet<String>,
    dependencies: BTreeSet<String>,
}

impl<'a> DependencyScanner<'a> {
    fn new(crate_dependencies: &'a BTreeSet<String>) -> Self {
        Self {
            crate_dependencies,
            dependencies: BTreeSet::new(),
        }
    }

    fn finish(self) -> BTreeSet<String> {
        self.dependencies
    }

    fn record(&mut self, crate_name: &syn::Ident) {
        let crate_name = crate_name.to_string();
        if self.crate_dependencies.contains(&crate_name) {
            self.dependencies.insert(crate_name);
        }
    }
}

impl Visit<'_> for DependencyScanner<'_> {
    fn visit_item_mod(&mut self, _item_mod: &syn::ItemMod) {}

    fn visit_item_use(&mut self, item_use: &syn::ItemUse) {
        match &item_use.tree {
            syn::UseTree::Path(path) => self.record(&path.ident),
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    if let syn::UseTree::Path(path) = item {
                        self.record(&path.ident);
                    };
                }
            }
            syn::UseTree::Name(name) => self.record(&name.ident),
            syn::UseTree::Rename(rename) => self.record(&rename.ident),
            _ => (),
        }
    }

    fn visit_path(&mut self, path: &syn::Path) {
        if let Some(head) = path.segments.first() {
            self.record(&head.ident);
        }
        syn::visit::visit_path(self, path);
    }
}

fn parse_ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str::<syn::Ident>(name).with_context(|| format!("parsing identifier `{name}`"))
}

fn public_module(name: &str, content: Vec<syn::Item>) -> Result<syn::ItemMod> {
    let ident = parse_ident(name)?;
    Ok(parse_quote!(pub mod #ident { #(#content)* }))
}

#[cfg(test)]
mod tests {
    use syn::visit::Visit;

    use super::DependencyScanner;

    fn test_scan(content: &str, crate_dependencies: &[&str], expected: &[&str]) {
        let items = syn::parse_file(content).expect("invalid testcase").items;
        let crate_dependencies = crate_dependencies
            .iter()
            .map(|dep| dep.to_string())
            .collect();
        let mut scanner = DependencyScanner::new(&crate_dependencies);
        items.iter().for_each(|item| scanner.visit_item(item));
        let dependencies = scanner.finish();
        let expected = expected.iter().map(|dep| dep.to_string()).collect();
        assert_eq!(dependencies, expected);
    }

    #[test]
    fn collect_use() {
        test_scan("use dep::Trait;", &["dep"], &["dep"]);
    }

    #[test]
    fn collect_grouped_use() {
        test_scan("use {dep::Trait, std::fmt::Debug};", &["dep"], &["dep"]);
    }

    #[test]
    fn collect_path() {
        test_scan("fn f() { dep::CONST }", &["dep"], &["dep"]);
    }

    #[test]
    fn collect_nested_path() {
        test_scan("type T = Vec<dep::Struct>;", &["dep"], &["dep"]);
    }

    #[test]
    fn ignore_dependency_in_child_module() {
        test_scan(" mod inner { use dep::Trait; }", &["dep"], &[]);
    }
}
