use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Lit, parse_macro_input, punctuated::Punctuated};

#[proc_macro]
pub fn gen_test(tokens: TokenStream) -> TokenStream {
    let punctuated =
        parse_macro_input!(tokens with Punctuated<Lit, syn::Token![,]>::parse_terminated);
    let punctuated = punctuated.iter().collect::<Vec<_>>();
    if punctuated.len() == 0 {
        panic!("One or two arguments expected");
    }

    let mut start = parse_lit_to_num(punctuated[0]);
    let mut end = punctuated
        .get(1)
        .map(|x| parse_lit_to_num(x))
        .unwrap_or(start);

    let mut prefix = false;
    if start == 0xCB {
        prefix = true;
        start = end;
        end = punctuated
            .get(2)
            .map(|x| parse_lit_to_num(x))
            .unwrap_or(start);
    }

    dbg!(start, end, prefix);
    let mut thing = Vec::with_capacity(usize::from(end - start));
    for i in start..=end {
        let name = format_ident!("opcode_{}0x{:02x}", if prefix { "cb_" } else { "" }, i);
        let file_path = format!(
            "./tests/sm83/v1/{}{:02x}.json",
            if prefix { "cb " } else { "" },
            i
        );
        thing.push(quote! {
            #[test]
            fn #name() {
                let tests = load_file(#file_path.to_owned());
                test_file(tests);
            }
        });
    }

    TokenStream::from(quote! {
        #(#thing)*
    })
}

fn parse_lit_to_num(lit: &Lit) -> u8 {
    let lit_int = match lit {
        Lit::Int(lit_int) => lit_int,
        _ => panic!("Literal must be an int"),
    };

    lit_int
        .base10_parse::<u8>()
        .expect("Unable to parse number")
}
