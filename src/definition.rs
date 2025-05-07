use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{Expr, FnArg, Ident, ItemFn, parse_quote, punctuated::Punctuated, token::Comma};

use crate::util::{all_args, get_meta_and_var_args, meta_args, var_args};

/// Given a signature, make identifiers “r_i” for each returned value.
fn return_identifiers(sig: &syn::Signature) -> Vec<Ident> {
    match &sig.output {
        syn::ReturnType::Default => Vec::new(),
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Tuple(tuple) => (0..tuple.elems.len())
                .map(|i| format_ident!("r_{}", i))
                .collect(),
            _ => vec![format_ident!("r_0")],
        },
    }
}

/// Given a list of idents, produce a matching pattern:
/// - 0 ⇒ `_`
/// - 1 ⇒ `r_0`
/// - n ⇒ `(r_0, r_1, …, r_{n-1})`
fn result_pattern(idents: &Vec<Ident>) -> syn::Pat {
    match idents.len() {
        0 => parse_quote! { _ },
        1 => {
            let id = &idents[0];
            parse_quote! { #id }
        }
        _ => parse_quote! { (#(#idents),*) },
    }
}

/// Given a function definition `definition` where:
///     - some arguments have type `var!(_)` ("ground" args)
///     - return values are all `Var`s
/// Produce a new function taking only the meta-arguments which constructs an OpenHypergraph
/// whose interfaces are defined by the ground arguments and return `Var` values.
pub fn generate_arrow_fn(
    definition: ItemFn,
    obj_type: Ident,
    arr_type: Ident,
    fn_name: Ident,
) -> ItemFn {
    let meta_and_ground = get_meta_and_var_args(&definition.sig);

    let meta_args: Punctuated<FnArg, Comma> = meta_args(&meta_and_ground);
    let all_args: Punctuated<Ident, Comma> = all_args(&meta_and_ground);
    let var_args: Vec<(Ident, TokenStream)> = var_args(&meta_and_ground);

    let builder_fn_name = definition.sig.ident.clone();

    let state_var_name = format_ident! { "state" };

    // declare the var args by making new vars for each builder
    // let v_i = Var::new(#builder_var_name)
    let var_arg_declarations: Vec<syn::Stmt> = var_args
        .clone()
        .into_iter()
        .map(|(ident, tokens)| {
            parse_quote! { let #ident = open_hypergraphs::lax::var::Var::new(#state_var_name.clone(), #tokens); }
        })
        .collect();

    // a "new source" statement like arg_i.new_source() for each var arg
    let var_arg_new_source_exprs: Vec<Expr> = var_args
        .into_iter()
        .map(|(ident, _)| {
            parse_quote! { #ident.new_source() }
        })
        .collect();

    // Assume result of annotated function is either unit type, single var, or a tuple of vars.
    // Get a list of identifiers, match pattern, and list of "ident.new_source()" expressions for
    // each returned var.
    let return_idents: Vec<syn::Ident> = return_identifiers(&definition.sig);
    let result_pattern = result_pattern(&return_idents);
    let result_pattern_new_target_exprs: Vec<Expr> = return_idents
        .into_iter()
        .map(|ident| {
            parse_quote! { #ident.new_source() }
        })
        .collect();

    // Generate a function taking the meta-arguments and producing an `OpenHypergraph` whose
    // sources are the `var!` annotated arguments, and whose targets are the vars returned from the
    // annotated function.
    parse_quote! {
        fn #fn_name(#meta_args) -> open_hypergraphs::lax::OpenHypergraph<#obj_type, #arr_type> {
            use std::vec::*;
            use open_hypergraphs::lax::*;

            let state = std::rc::Rc::new(std::cell::RefCell::new(OpenHypergraph::<#obj_type, #arr_type>::empty()));

            {
                // Declare "var" args
                #( #var_arg_declarations )*

                // Create a new source node for each Var.
                // We do this before calling the builder function in case it takes ownership.
                state.borrow_mut().sources = vec![
                    #(#var_arg_new_source_exprs),*
                ];

                // Call the builder function with meta and var args
                let result = #builder_fn_name(state.clone(), #all_args);

                state.borrow_mut().targets = {
                    let #result_pattern = result;
                    vec![#(#result_pattern_new_target_exprs),*]
                }
            }

            std::rc::Rc::try_unwrap(state).unwrap().into_inner()
        }
    }
}
