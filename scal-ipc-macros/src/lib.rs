use proc_macro::TokenStream;
use proc_macro2::{Delimiter, LineColumn, TokenTree};
use quote::quote;
use syn::{Expr, ItemFn, parse_macro_input, spanned::Spanned};

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

/// Keywords that can legitimately be followed (or preceded) by another
/// identifier inside a single expression, so they never start/end an item.
const NO_SPLIT_AFTER: &[&str] = &[
    "if", "else", "match", "move", "return", "let", "in", "as", "async", "unsafe", "while", "for",
    "loop", "break", "continue", "mut", "ref", "dyn", "const", "static", "yield", "await",
    "box", "become", "try",
];
const NO_SPLIT_BEFORE: &[&str] = &["as", "else", "in", "await"];

fn ends_item(tt: &TokenTree) -> bool {
    match tt {
        TokenTree::Ident(i) => !NO_SPLIT_AFTER.contains(&i.to_string().as_str()),
        TokenTree::Literal(_) => true,
        // `else` / `as` are excluded on the `starts_item` side.
        TokenTree::Group(g) => g.delimiter() != Delimiter::None,
        TokenTree::Punct(_) => false,
    }
}

fn starts_item(tt: &TokenTree) -> bool {
    match tt {
        TokenTree::Ident(i) => !NO_SPLIT_BEFORE.contains(&i.to_string().as_str()),
        TokenTree::Literal(_) => true,
        _ => false,
    }
}

/// Splits the input into items. Items are separated by top-level commas, but
/// a missing comma is tolerated: two adjacent tokens that can never belong to
/// the same expression (e.g. `foo()` directly followed by `bar`) also start a
/// new item. This keeps the macro expandable while the user is still typing,
/// which lets rust-analyzer offer completions without typing the comma first.
fn split_items(input: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut items = Vec::new();
    let mut cur: Vec<TokenTree> = Vec::new();

    let flush = |cur: &mut Vec<TokenTree>, items: &mut Vec<proc_macro2::TokenStream>| {
        if !cur.is_empty() {
            items.push(cur.drain(..).collect());
        }
    };

    for tt in input {
        if let TokenTree::Punct(p) = &tt {
            if p.as_char() == ',' {
                flush(&mut cur, &mut items);
                continue;
            }
        }
        if let Some(prev) = cur.last() {
            if ends_item(prev) && starts_item(&tt) {
                flush(&mut cur, &mut items);
            }
        }
        cur.push(tt);
    }
    flush(&mut cur, &mut items);
    items
}

#[proc_macro]
pub fn timeline(input: TokenStream) -> TokenStream {
    let mut out = proc_macro2::TokenStream::new();

    for chunk in split_items(input.into()) {
        let expr = match syn::parse2::<Expr>(chunk) {
            Ok(e) => e,
            Err(err) => {
                // Report the broken item but keep expanding the others.
                out.extend(err.to_compile_error());
                continue;
            }
        };
        let sp: proc_macro2::Span = expr.span();
        let lc: LineColumn = sp.start();
        let line = lc.line as u32;

        out.extend(quote! {
            {
                let __op = ::scal_core::IntoAnimOp::into_anim_op(#expr);
                __op.with_location(::scal_core::SourceLoc {
                    file: file!().to_string(),
                    line: #line,
                })
            },
        });
    }

    quote! {
        vec![ #out ]
    }
    .into()
}
