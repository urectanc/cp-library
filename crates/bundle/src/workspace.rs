use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, ensure};
use cargo_toml::Manifest;

pub struct Workspace {
    crates: BTreeMap<String, Crate>,
}

impl Workspace {
    pub fn load(workspace_dir: impl AsRef<Path>) -> Result<Self> {
        let workspace_dir = workspace_dir.as_ref().canonicalize()?;
        let manifest = Manifest::from_path(workspace_dir.join("Cargo.toml"))
            .context("loading workspace manifest")?;
        let workspace_deps = &manifest
            .workspace
            .as_ref()
            .ok_or_else(|| anyhow!("not a Cargo workspace"))?
            .dependencies;

        let crate_dirs = workspace_deps
            .iter()
            .filter_map(|(name, deps)| {
                let relative_path = deps.detail()?.path.as_deref()?;
                let crate_dir = workspace_dir.join(relative_path).canonicalize().ok()?;
                crate_dir
                    .starts_with(&workspace_dir)
                    .then_some((name, crate_dir))
            })
            .collect::<BTreeMap<_, _>>();

        let crates = crate_dirs
            .iter()
            .map(|(name, crate_dir)| {
                let manifest = Manifest::from_path(crate_dir.join("Cargo.toml"))
                    .with_context(|| format!("loading crate manifest `{name}`"))?;
                let package = manifest
                    .package
                    .as_ref()
                    .ok_or_else(|| anyhow!("not a Cargo package `{name}`"))?;

                // https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml
                ensure!(
                    package.name() == name.as_str(),
                    "renaming dependencies `{}` to `{}` not supported",
                    package.name(),
                    name
                );
                let crate_name = String::from(package.name());

                let dependencies = manifest
                    .dependencies
                    .iter()
                    .filter_map(|(name, dep)| {
                        // foo.workspace = true
                        let detail = dep.detail()?;
                        (detail.inherited && crate_dirs.contains_key(name))
                            .then_some(String::from(name.as_str()))
                    })
                    .collect();

                Ok((
                    crate_name.clone(),
                    Crate::new(crate_name, crate_dir.to_owned(), dependencies),
                ))
            })
            .collect::<Result<_>>()?;

        Ok(Workspace { crates })
    }

    pub fn list(&self) -> Vec<String> {
        self.crates.keys().map(String::from).collect()
    }

    pub fn expand(&self, crates: &[impl AsRef<str>], namespace: &str) -> Result<String> {
        let crates = crates
            .iter()
            .map(|crate_name| String::from(crate_name.as_ref()))
            .collect::<Vec<_>>();

        let expanded_crates = self
            .resolve(&crates)?
            .iter()
            .map(|name| self.get_crate(name).unwrap())
            .collect();
        let mut ast = crate::build::build(expanded_crates, namespace)?;
        crate::refine::refine(&mut ast);
        Ok(prettyplease::unparse(&ast))
    }

    pub fn bundle(&self, source: &str, namespace: &str) -> Result<String> {
        let used_names = crate::source::names_under_namespace(source, namespace)?;
        if used_names.is_empty() {
            return Ok(String::new());
        }

        self.expand(&used_names, namespace)
    }

    fn get_crate(&self, crate_name: &str) -> Option<&Crate> {
        self.crates.get(crate_name)
    }

    fn resolve(&self, crates: &[String]) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        for crate_name in crates {
            if seen.insert(crate_name) {
                queue.push_back(crate_name);
            }
        }

        while let Some(crate_name) = queue.pop_front() {
            resolved.push(crate_name.clone());
            let dependencies = self
                .get_crate(crate_name)
                .with_context(|| format!("unknown crate `{crate_name}`"))?
                .dependencies();
            for dependency in dependencies {
                if seen.insert(dependency) {
                    queue.push_back(dependency);
                }
            }
        }

        Ok(resolved)
    }
}

pub(crate) struct Crate {
    name: String,
    root_dir: PathBuf,
    dependencies: BTreeSet<String>,
}

impl Crate {
    fn new(name: String, root_dir: PathBuf, dependencies: BTreeSet<String>) -> Self {
        Self {
            name,
            root_dir,
            dependencies,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn src_dir(&self) -> PathBuf {
        self.root_dir.join("src")
    }

    pub(crate) fn dependencies(&self) -> &BTreeSet<String> {
        &self.dependencies
    }
}
