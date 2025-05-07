//! Utilities to help construct:
//!     1. A list of identifiers for each return value from the builder function
//!     2. An "unpacking expression" like `let (r_0, r_1) = ..` from the builder function call
use quote::format_ident;
use syn::{Ident, Pat, ReturnType, Signature, Type, parse_quote};

/// Given a signature, make identifiers “r_i” for each returned value.
pub(crate) fn return_identifiers(sig: &Signature) -> Vec<Ident> {
    match &sig.output {
        ReturnType::Default => Vec::new(),
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) => (0..tuple.elems.len())
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
pub(crate) fn result_pattern(idents: &Vec<Ident>) -> Pat {
    match idents.len() {
        0 => parse_quote! { _ },
        1 => {
            let id = &idents[0];
            parse_quote! { #id }
        }
        _ => parse_quote! { (#(#idents),*) },
    }
}
