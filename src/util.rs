use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{FnArg, Ident, Pat, Signature, Type, TypeMacro, punctuated::Punctuated, token::Comma};

// Try to use `fresh` as a fresh name, prefixing with underscores until it doesn't shadow an
// identifier in `vars_in_scope`.
pub fn fresh_name(mut fresh: String, vars_in_scope: Vec<Ident>) -> Ident {
    // Get the sets of identifiers as strings.
    let vars_set: HashSet<String> = vars_in_scope
        .iter()
        .map(|ident| ident.to_string())
        .collect();

    // Keep adding underscores until we find a name that doesn't shadow
    while vars_set.contains(&fresh) {
        fresh = format!("_{}", fresh);
    }

    Ident::new(&fresh, proc_macro2::Span::call_site())
}

/// The `var!(value)` macro allows users to write rust values (`value`) in type positions. For
/// example,
/// `fn foo(x: var!(42))`.
///
/// The `MixedType` enum records if a type is either:
///     1. A genuine type
///     2. A `var!` macro value
#[derive(Clone)]
pub enum MixedType {
    #[allow(dead_code)] // allow the unused syn::Type here.
    Meta(syn::Type), // Metavariables just have a type.
    Var(TokenStream), // Ground variables have a *value* - the label in the open hypergraph.
}

/// Check if a syn::Type is a `var!` macro.
pub fn type_to_mixed_type(ty: &syn::Type) -> MixedType {
    match ty {
        // If we had a type macro `var!(..some tokens..)`, return `MixedType::Var` with those tokens.
        // Otherwise, leave the type unchanged, and return `MixedType::Meta`
        Type::Macro(TypeMacro { mac }) => {
            if mac.path.is_ident("var") {
                MixedType::Var(mac.tokens.clone())
            } else {
                MixedType::Meta(ty.clone())
            }
        }

        _ => MixedType::Meta(ty.clone()),
    }
}

pub fn pat_to_ident(pat: &syn::Pat) -> syn::Ident {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        _ => panic!("TODO"),
    }
}

pub fn pat_type_to_mixed(pat_type: &syn::PatType) -> (Ident, MixedType) {
    let ident = pat_to_ident(&pat_type.pat);
    let mixed_type = type_to_mixed_type(&pat_type.ty);
    (ident, mixed_type)
}

/// Given a function signature, make a list of input identifiers and their types interpreted as a
/// `MixedType`.
pub fn get_meta_and_var_args(sig: &Signature) -> Vec<(Ident, MixedType)> {
    let mut result = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => panic!("TODO"),
            FnArg::Typed(pat_type) => {
                result.push(pat_type_to_mixed(pat_type));
            }
        }
    }
    result
}

/// Filter out the "meta" arguments from a list of identifiers and their `MixedType`.
pub fn meta_args(args: &Vec<(Ident, MixedType)>) -> Punctuated<Ident, Comma> {
    let mut result = Punctuated::new();
    for (ident, ty) in args {
        if let MixedType::Meta(_) = ty {
            result.push(ident.clone())
        }
    }
    result
}

/// Return a ,-puncutated list of identifiers
pub fn all_args(args: &Vec<(Ident, MixedType)>) -> Punctuated<Ident, Comma> {
    let mut result = Punctuated::new();
    for (ident, _) in args {
        result.push(ident.clone())
    }
    result
}

/// Filter out the "var" (or "ground") arguments from a list of identifiers and their `MixedType`.
pub fn var_args(args: &Vec<(Ident, MixedType)>) -> Vec<(Ident, TokenStream)> {
    let mut result = Vec::new();
    for (ident, ty) in args {
        if let MixedType::Var(tokens) = ty {
            result.push((ident.clone(), tokens.clone()))
        }
    }
    result
}
