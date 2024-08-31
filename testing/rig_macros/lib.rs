use proc_macro::TokenStream;

use anyhow::{bail, Context};
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

    let mut setup_id = None;

    for (id, item) in items.iter_mut().enumerate() {
        if let &mut Item::Fn(ref mut fun) = item {
            let pos = fun.attrs.iter().position(|a| has_ident(a, "case"));
            if let Some(pos) = pos {
                cases.push(fun.sig.ident.clone());
                fun.attrs.remove(pos);
            } else if fun.attrs.iter().any(|a| has_ident(a, "setup")) {
                setup_id = Some(id);
            }
        }
    }

    let setup: Item = items.swap_remove(setup_id.context("No setup function found")?);
    let input_arg = if let Item::Fn(setup) = &setup {
        if setup.sig.inputs.len() != 1 {
            bail!("Setup function must take exactly one argument");
        } else {
            setup.sig.inputs.first().unwrap().clone()
        }
    } else {
        bail!("Setup is not a function");
    };
    let input_name = match &input_arg {
        syn::FnArg::Typed(pat) => match &*pat.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => bail!("Setup argument must be an identifier"),
        },
        _ => bail!("Setup argument cannot be self"),
    };
    let case_names = cases.iter().map(|c| c.to_string());

    let name = &res.ident;
    let name_str = name.to_string();

    let new_item = quote! {
        pub fn run(#input_arg) {
            runner::run_tests(runner::Test::Suite {
                name: #name_str.to_string(),
                tests: vec![
                    #(
                        runner::Test::Case {
                            name: #case_names.to_string(),
                            code: Box::new(|#input_name| {
                                #cases(#input_name);
                                Ok(())
                            }),
                        }
                    ),*
                ],
            }, #input_name);
        }
    };

    Ok(quote! {
        pub mod #name {
            #(#items)*
            #new_item
        }
    })
}

fn has_ident(attr: &Attribute, ident_name: &str) -> bool {
    match &attr.meta {
        Meta::Path(p) => match p.require_ident() {
            Ok(i) => *i == ident_name,
            Err(_) => false,
        },
        _ => false,
    }
}
