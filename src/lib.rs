mod definition;
mod fn_args;
mod fn_returns;
mod var;

extern crate proc_macro;
use crate::definition::generate_arrow_fn;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{Ident, ItemFn, Token, parse_macro_input, punctuated::Punctuated};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn def_arrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse the attribute arguments: (ObjType, ArrType, new_fn_name)
    let args = parse_macro_input!(attr with Punctuated::<Ident, Token![,]>::parse_terminated);
    let mut iter = args.into_iter();
    let obj_type = iter.next().expect("expected ObjType");
    let arr_type = iter.next().expect("expected ArrType");
    let fn_name = iter.next().expect("expected new function name");

    // 2. Parse the original function
    let input_fn: ItemFn = parse_macro_input!(item as ItemFn);

    // 3. Generate the new arrow function
    let generated_fn = generate_arrow_fn(
        input_fn.clone(),
        obj_type.clone(),
        arr_type.clone(),
        fn_name,
    );

    // 4. Modify original function to replace var! with Var<O, A>.
    let input_fn = var::expand_var_macros(input_fn, obj_type.clone(), arr_type.clone());

    // 5. Combine original + generated
    let out = quote! {
        // original function, made "stateful"
        #[state_macro::stateful_cloned(std::rc::Rc<std::cell::RefCell<open_hypergraphs::lax::OpenHypergraph<#obj_type, #arr_type>>>)]
        #input_fn

        // Generated arrow definition
        #generated_fn
    };

    out.into()
}
