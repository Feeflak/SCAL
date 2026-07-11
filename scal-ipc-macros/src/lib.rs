use proc_macro::TokenStream;
use proc_macro2::LineColumn;
use quote::quote;
use syn::{
    Expr, ItemFn, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
};

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

struct TimelineInput {
    items: Punctuated<Expr, Token![,]>,
}

impl Parse for TimelineInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TimelineInput {
            items: input.parse_terminated(Expr::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn timeline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TimelineInput);

    let mut out = proc_macro2::TokenStream::new();

    for item in &input.items {
        let expr = item;
        let sp: proc_macro2::Span = expr.span();
        let lc: LineColumn = sp.start();
        let line = lc.line as u32;
        let col = lc.column as u32 + 1;

        out.extend(quote! {
            {
                let __op = ::scal_core::IntoAnimOp::into_anim_op(#expr);
                __op.with_location(::scal_core::SourceLoc {
                    file: file!().to_string(),
                    line: #line,
                    col: #col,
                })
            },
        });
    }

    quote! {
        vec![ #out ]
    }
    .into()
}
