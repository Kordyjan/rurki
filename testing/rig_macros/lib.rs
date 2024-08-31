use proc_macro::TokenStream;

use anyhow::Context;
use quote::quote;
use syn::{Attribute, Ident, Item, ItemMod, Meta};

#[proc_macro_attribute]
pub fn test_suite(_params: TokenStream, body: TokenStream) -> TokenStream {
    match test_suite_impl(body) {
        Err(e) => panic!("{:?}", e),
        Ok(ts) => ts.into(),
    }
}

fn test_suite_impl(body: TokenStream) -> anyhow::Result<proc_macro2::TokenStream> {
    let mut res: ItemMod = syn::parse(body)?;
    let (_, items) = (res.content).as_mut().context("Module without body")?;

    let mut cases = Vec::<Ident>::new();

    for item in items.iter_mut() {
        if let &mut Item::Fn(ref mut fun) = item {
            let pos = fun.attrs.iter().position(is_case);
            if let Some(pos) = pos {
                cases.push(fun.sig.ident.clone());
                fun.attrs.remove(pos);
            }
        }
    }

    let case_names = cases.iter().map(|c| c.to_string());

    let name = &res.ident;
    let name_str = name.to_string();

    let new_item = quote! {
        pub fn run() {
            runner::run_tests(runner::Test::Suite {
                name: #name_str.to_string(),
                tests: vec![
                    #(
                        runner::Test::Case {
                            name: #case_names.to_string(),
                            code: Box::new(|| {
                                #cases();
                                Ok(())
                            }),
                        }
                    ),*
                ],
            });
        }
    };

    // panic!("{}", new_item.to_string());

    Ok(quote! {
        pub mod #name {
            #(#items)*
            #new_item
        }
    })
}

fn is_case(attr: &Attribute) -> bool {
    match &attr.meta {
        Meta::Path(p) => match p.require_ident() {
            Ok(i) => *i == "case",
            Err(_) => false,
        },
        _ => false,
    }
}
