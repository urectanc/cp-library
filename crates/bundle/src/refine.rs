use syn::visit_mut::VisitMut;

pub(crate) fn refine(ast: &mut syn::File) {
    remove_doc_attrs(&mut ast.attrs);
    refine_items(&mut ast.items);
}

fn refine_items(items: &mut Vec<syn::Item>) {
    items.retain_mut(|item| !is_test_item(item_attrs(item)));
    for item in items {
        Refine.visit_item_mut(item);
    }
}

fn refine_trait_items(items: &mut Vec<syn::TraitItem>) {
    items.retain_mut(|item| !is_test_item(trait_item_attrs(item)));
    for item in items {
        Refine.visit_trait_item_mut(item);
    }
}

fn refine_impl_items(items: &mut Vec<syn::ImplItem>) {
    items.retain_mut(|item| !is_test_item(impl_item_attrs(item)));
    for item in items {
        Refine.visit_impl_item_mut(item);
    }
}

fn refine_foreign_items(items: &mut Vec<syn::ForeignItem>) {
    items.retain_mut(|item| !is_test_item(foreign_item_attrs(item)));
    for item in items {
        Refine.visit_foreign_item_mut(item);
    }
}

struct Refine;

impl VisitMut for Refine {
    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        remove_doc_attrs(item_attrs(item));

        match item {
            syn::Item::ForeignMod(item) => refine_foreign_items(&mut item.items),
            syn::Item::Impl(item) => refine_impl_items(&mut item.items),
            syn::Item::Mod(item) => {
                if let Some((_, children)) = &mut item.content {
                    refine_items(children);
                }
            }
            syn::Item::Trait(item) => refine_trait_items(&mut item.items),
            _ => (),
        }
    }

    fn visit_trait_item_mut(&mut self, item: &mut syn::TraitItem) {
        remove_doc_attrs(trait_item_attrs(item));
    }

    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        remove_doc_attrs(impl_item_attrs(item));
    }

    fn visit_foreign_item_mut(&mut self, item: &mut syn::ForeignItem) {
        remove_doc_attrs(foreign_item_attrs(item));
    }
}

fn is_test_item(attrs: &mut [syn::Attribute]) -> bool {
    attrs.iter().any(is_test_attr)
}

fn is_test_attr(attr: &syn::Attribute) -> bool {
    match &attr.meta {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(meta_list) => {
            meta_list.path.is_ident("cfg")
                && meta_list
                    .parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))
        }
        _ => false,
    }
}

fn remove_doc_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !attr.path().is_ident("doc"));
}

fn item_attrs(item: &mut syn::Item) -> &mut Vec<syn::Attribute> {
    match item {
        syn::Item::Const(item) => &mut item.attrs,
        syn::Item::Enum(item) => &mut item.attrs,
        syn::Item::ExternCrate(item) => &mut item.attrs,
        syn::Item::Fn(item) => &mut item.attrs,
        syn::Item::ForeignMod(item) => &mut item.attrs,
        syn::Item::Impl(item) => &mut item.attrs,
        syn::Item::Macro(item) => &mut item.attrs,
        syn::Item::Mod(item) => &mut item.attrs,
        syn::Item::Static(item) => &mut item.attrs,
        syn::Item::Struct(item) => &mut item.attrs,
        syn::Item::Trait(item) => &mut item.attrs,
        syn::Item::TraitAlias(item) => &mut item.attrs,
        syn::Item::Type(item) => &mut item.attrs,
        syn::Item::Union(item) => &mut item.attrs,
        syn::Item::Use(item) => &mut item.attrs,
        syn::Item::Verbatim(_) => unimplemented!(),
        _ => unreachable!(),
    }
}

fn trait_item_attrs(item: &mut syn::TraitItem) -> &mut Vec<syn::Attribute> {
    match item {
        syn::TraitItem::Const(item) => &mut item.attrs,
        syn::TraitItem::Fn(item) => &mut item.attrs,
        syn::TraitItem::Macro(item) => &mut item.attrs,
        syn::TraitItem::Type(item) => &mut item.attrs,
        syn::TraitItem::Verbatim(_) => unimplemented!(),
        _ => unreachable!(),
    }
}

fn impl_item_attrs(item: &mut syn::ImplItem) -> &mut Vec<syn::Attribute> {
    match item {
        syn::ImplItem::Const(item) => &mut item.attrs,
        syn::ImplItem::Fn(item) => &mut item.attrs,
        syn::ImplItem::Macro(item) => &mut item.attrs,
        syn::ImplItem::Type(item) => &mut item.attrs,
        syn::ImplItem::Verbatim(_) => unimplemented!(),
        _ => unreachable!(),
    }
}

fn foreign_item_attrs(item: &mut syn::ForeignItem) -> &mut Vec<syn::Attribute> {
    match item {
        syn::ForeignItem::Fn(item) => &mut item.attrs,
        syn::ForeignItem::Macro(item) => &mut item.attrs,
        syn::ForeignItem::Static(item) => &mut item.attrs,
        syn::ForeignItem::Type(item) => &mut item.attrs,
        syn::ForeignItem::Verbatim(_) => unimplemented!(),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn remove_tests() {
        let refined = refine_and_render(
            r#"
            #[test]
            fn remove_test_function() {}

            #[cfg(test)]
            mod remove_test_module {
                pub fn helper() {}
            }

            mod keep_module {
                #[cfg(test)]
                pub fn remove_nested_test_function() {}

                pub fn keep_nested_function() {}
            }

            pub fn keep_function() {}
            "#,
        );

        assert!(refined.contains("mod keep_module"));
        assert!(refined.contains("pub fn keep_nested_function()"));
        assert!(refined.contains("pub fn keep_function()"));
        assert!(!refined.contains("remove_test_function"));
        assert!(!refined.contains("remove_test_module"));
        assert!(!refined.contains("remove_nested_test_function"));
    }

    #[test]
    fn remove_doc_comments() {
        let refined = refine_and_render(
            r#"
            //! REMOVE_INNER_DOC

            /// REMOVE_STRUCT_DOC
            pub struct Keep;

            /// REMOVE_TRAIT_DOC
            pub trait Trait {
                /// REMOVE_TRAIT_CONST_DOC
                const VALUE: u32;
            }

            /// REMOVE_IMPL_DOC
            impl Keep {
                /// REMOVE_IMPL_FN_DOC
                pub fn keep() {}
            }
            "#,
        );

        assert!(refined.contains("pub struct Keep;"));
        assert!(refined.contains("pub trait Trait"));
        assert!(refined.contains("const VALUE: u32;"));
        assert!(refined.contains("pub fn keep()"));
        assert!(!refined.contains("REMOVE_INNER_DOC"));
        assert!(!refined.contains("REMOVE_STRUCT_DOC"));
        assert!(!refined.contains("REMOVE_TRAIT_DOC"));
        assert!(!refined.contains("REMOVE_TRAIT_CONST_DOC"));
        assert!(!refined.contains("REMOVE_IMPL_DOC"));
        assert!(!refined.contains("REMOVE_IMPL_FN_DOC"));
    }

    fn refine_and_render(source: &str) -> String {
        let mut ast = syn::parse_file(source).expect("invalid testcase");
        super::refine(&mut ast);
        prettyplease::unparse(&ast)
    }
}
