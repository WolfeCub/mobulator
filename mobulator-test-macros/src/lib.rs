use proc_macro::TokenStream;
use syn::{parse_macro_input, punctuated::Punctuated, Lit};
use quote::{format_ident, quote};

#[proc_macro]
pub fn gen_test(tokens: TokenStream) -> TokenStream {
    let punctuated = parse_macro_input!(tokens with Punctuated<Lit, syn::Token![,]>::parse_terminated);
    let punctuated = punctuated.iter().collect::<Vec<_>>();
    if punctuated.len() == 0 {
        panic!("One or two arguments expected");
    }

    let start = parse_lit_to_num(punctuated[0]);
    let end = if let Some(e) = punctuated.get(1) {
        parse_lit_to_num(e)
    } else {
        start
    };

    let mut thing = Vec::with_capacity(usize::from(end-start));
    for i in start..=end {
        let name = format_ident!("opcode_0x{:02x}", i);
        thing.push(quote! {
            #[test]
            fn #name() {
                let file_name = format!("./tests/sm83/v1/{:02x}.json", #i);
                let tests = load_file(file_name);
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

    lit_int.base10_parse::<u8>().expect("Unable to parse number")
}
