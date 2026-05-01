# bundle

[Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) として分割管理されているライブラリから、指定された crate とその依存関係だけを 1 つの module として出力する。

## API

```rust
let workspace = bundle::Workspace::load("/path/to/library")?;

// list bundle-able crates
let list = workspace.list();
assert_eq!(list, vec![String::from("entry")]);

// specify crates
let rendered = workspace.expand(&["entry"], "bundled")?;

// or infer crates from source
let rendered = workspace.bundle(
    "fn main() { let _ = bundled::entry::Item; }",
    "bundled",
)?;

assert_eq!(
    rendered,
    r#"pub mod bundled {
    pub mod entry {
        // ...
    }
}
"#
);
```


## 仕様

bundle 対象とする crate は workspace root 以下に置き、 `workspace.dependencies` に [path dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies) として列挙する。
crate 間の依存関係は、各 crate の `Cargo.toml` に [workspace dependency を継承する形](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace)で記述する。

bundle に渡す source では、 bundle 対象 crate を `<namespace>::<crate>` の形で参照する。
`use <namespace>::*` のような glob import は、出力が巨大になりすぎるため非対応としている。

出力からは通常コメントと doc comment 、および`#[test]` または `#[cfg(test)]` が付いた item が除去される。

### 例

```text
library/
├── Cargo.toml
├── entry/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── dependency/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

```toml
# Cargo.toml

[workspace]
members = ["entry", "dependency"]

[workspace.dependencies]
entry = { path = "./entry" }
dependency = { path = "./dependency" }
```

```toml
# entry/Cargo.toml

[package]
name = "entry"
# ...

[dependencies]
dependency.workspace = true
```
