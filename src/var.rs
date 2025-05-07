/// Utilities to expand `var!` macros in a `def_arrow`-annotated function.
use syn::parse_quote;
use syn::{FnArg, Ident, ItemFn, PatType, Signature, Type, TypeMacro};

/// Recursively expand all appearances of var!(..) within a syn::Type
/// into Var<obj_type, arr_type>.
fn recursively_expand_type(ty: syn::Type, obj_type: &Ident, arr_type: &Ident) -> syn::Type {
    let expanded = parse_quote! { open_hypergraphs::lax::var::Var<#obj_type, #arr_type> };
    match ty {
        Type::Macro(TypeMacro { ref mac, .. }) => {
            if mac.path.is_ident("var") {
                expanded
            } else {
                ty
            }
        }
        Type::Tuple(syn::TypeTuple { paren_token, elems }) => {
            let elems = elems
                .iter()
                .map(|t| recursively_expand_type(t.clone(), obj_type, arr_type))
                .collect();
            Type::Tuple(syn::TypeTuple { elems, paren_token })
        }
        _ => ty,
    }
}

fn expand_type(ty: &mut Box<syn::Type>, obj_type: &Ident, arr_type: &Ident) {
    if let Type::Macro(TypeMacro { mac, .. }) = &**ty {
        if mac.path.is_ident("var") {
            *ty = parse_quote! { open_hypergraphs::lax::var::Var<#obj_type, #arr_type> };
        }
    }
}

fn expand_input_vars(sig: &mut Signature, obj_type: Ident, arr_type: Ident) {
    for fn_arg in sig.inputs.iter_mut() {
        if let FnArg::Typed(PatType { ty, .. }) = fn_arg {
            expand_type(ty, &obj_type, &arr_type);
        }
    }
}

/// Recursively expand output variables.
fn expand_output_vars(sig: &mut Signature, obj_type: Ident, arr_type: Ident) {
    if let syn::ReturnType::Type(_, ref mut ty) = sig.output {
        *ty = Box::new(recursively_expand_type(*ty.clone(), &obj_type, &arr_type));
    }
}

// Modify a signature in-place by expanding input and output vars.
fn expand_signature(sig: &mut Signature, obj_type: Ident, arr_type: Ident) {
    expand_input_vars(sig, obj_type.clone(), arr_type.clone());
    expand_output_vars(sig, obj_type, arr_type);
}

/// Expand all var! macros in the function signature into Var<obj_type, arr_type>.
pub(crate) fn expand_var_macros(item_fn: ItemFn, obj_type: Ident, arr_type: Ident) -> ItemFn {
    let mut item_fn = item_fn.clone();
    expand_signature(&mut item_fn.sig, obj_type, arr_type);
    item_fn
}
