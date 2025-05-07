//! Utilities to extract information from the arguments to a function.
//! Arguments are represented internally as a pair of Ident, MixedType.
//! The latter records simplify that a type was either a rust type, or the macro `var!(..)`.
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{
    FnArg, Ident, Pat, Signature, Type, TypeMacro, parse_quote, punctuated::Punctuated,
    token::Comma,
};

/// The `var!(value)` macro allows users to write rust values (`value`) in type positions. For
/// example,
/// `fn foo(x: var!(42))`.
///
/// The `MixedType` enum records if a type is either:
///     1. A genuine type
///     2. A `var!` macro value
#[derive(Clone)]
pub enum MixedType {
    Meta(syn::Type),  // Metavariables just have a type.
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

pub fn pat_to_ident(pat: &syn::Pat, arg_index: usize) -> syn::Ident {
    match pat {
        Pat::Ident(pat_ident) => format_ident!("arg_{}_{}", arg_index, pat_ident.ident.clone()),
        _ => format_ident!("arg_{}_pattern", arg_index),
    }
}

pub fn pat_type_to_mixed(pat_type: &syn::PatType, arg_index: usize) -> (Ident, MixedType) {
    let ident = pat_to_ident(&pat_type.pat, arg_index);
    let mixed_type = type_to_mixed_type(&pat_type.ty);
    (ident, mixed_type)
}

/// Given a function signature, make a list of input identifiers and their types interpreted as a
/// `MixedType`.
pub fn get_meta_and_var_args(sig: &Signature) -> Vec<(Ident, MixedType)> {
    let mut result = Vec::new();
    for (arg_index, arg) in sig.inputs.iter().enumerate() {
        match arg {
            FnArg::Receiver(_) => panic!("TODO"),
            FnArg::Typed(pat_type) => {
                result.push(pat_type_to_mixed(pat_type, arg_index));
            }
        }
    }
    result
}

/// Filter out the "meta" arguments from a list of identifiers and their `MixedType`.
pub fn meta_args(args: &Vec<(Ident, MixedType)>) -> Punctuated<FnArg, Comma> {
    let mut result = Punctuated::new();
    for (ident, ty) in args {
        if let MixedType::Meta(arg_type) = ty {
            let fn_arg: FnArg = parse_quote! { #ident: #arg_type };
            result.push(fn_arg)
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
