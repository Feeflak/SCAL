use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_block = &input_fn.block;

    let expanded = quote! {
        fn animation_main() -> ::scal_core::Project #fn_block

        fn main() {
            ::scal_ipc::run_main(animation_main);
        }
    };

    expanded.into()
}
