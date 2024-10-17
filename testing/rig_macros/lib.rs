use std::collections::HashSet;

use proc_macro::TokenStream;

use anyhow::{bail, Context};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Attribute, Expr, FnArg, GenericParam, Ident, Item, ItemMod, Meta, Stmt
};

#[proc_macro_attribute]
pub fn test_suite(_params: TokenStream, body: TokenStream) -> TokenStream {
    match test_suite_impl(body) {
        Err(e) => panic!("{e:?}"),
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

    let (input_generics, input_arg) = if let Item::Fn(setup) = &setup {
        let generics = setup.sig.generics.clone();
        if setup.sig.inputs.len() != 1 {
            bail!("Setup function must take exactly one argument");
        }
        let arg = setup.sig.inputs.first().unwrap().clone();
        (generics, arg)
    } else {
        bail!("Setup is not a function");
    };

    let (input_name, input_type) = match &input_arg {
        syn::FnArg::Typed(pat) => {
            let name = match &*pat.pat {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => bail!("Setup argument must be an identifier"),
            };
            let tpe = pat.ty.clone();
            (name, tpe)
        }
        FnArg::Receiver(_) => bail!("Setup argument cannot be self"),
    };
    let case_names = cases.iter().map(ToString::to_string).collect::<Vec<_>>();
    let case_set: HashSet<String> = case_names.iter().cloned().collect();

    let setup_body = if let Item::Fn(setup) = setup {
        setup.block.stmts
    } else {
        bail!("Setup is not a function");
    };

    for item in items.iter_mut() {
        if let Item::Fn(fun) = item {
            if case_set.contains(&fun.sig.ident.to_string()) {
                let sig = &mut fun.sig;
                sig.inputs.insert(0, input_arg.clone());
                sig.generics = input_generics.clone();
                sig.output = parse_quote! { -> runner::Result };

                let mut tmp = setup_body.clone();
                if let Some(Stmt::Expr(_, sem)) = fun.block.stmts.last_mut() {
                    *sem = Some(parse_quote! {;});
                }
                tmp.append(&mut fun.block.stmts);
                tmp.push(Stmt::Expr(Expr::Verbatim(quote! { Ok(()) }), None));
                fun.block.stmts = tmp;
            }
        }
    }

    let name = &res.ident;
    let name_str = name.to_string();
    let lt = input_generics.lt_token;
    let params = input_generics.params.clone();
    let gt = input_generics.gt_token;
    let where_clause = input_generics.where_clause.clone();

    let params_bare = params
        .clone()
        .into_iter()
        .map(|par| match par {
            GenericParam::Type(t) => t.ident.clone(),
            GenericParam::Lifetime(l) => l.lifetime.ident.clone(),
            GenericParam::Const(c) => c.ident.clone(),
        })
        .collect::<Punctuated<_, Comma>>();

    let double_colon = lt.as_ref().map(|_| quote! { :: });

    let suite_item = quote! {
        pub fn suite #lt #params #gt () -> runner::model::Test<#input_type> #where_clause {
            runner::model::Test::Suite {
                name: #name_str.to_string(),
                tests: vec![
                    #(
                        runner::model::Test::Case {
                            name: #case_names.to_string(),
                            code: Box::new(|#input_arg| {
                                #cases #double_colon #lt #params_bare #gt (#input_name)
                            }),
                        }
                    ),*
                ],
            }
        }
    };

    // bail!("{}", items[1].to_token_stream());

    Ok(quote! {
        pub mod #name {
            #(#items)*
            #suite_item
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
