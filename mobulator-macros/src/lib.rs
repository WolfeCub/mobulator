use proc_macro::TokenStream;
use quote::quote;
use syn::{Lit, parse_macro_input};

#[proc_macro]
pub fn opcode_match(tokens: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(tokens as Lit);
    let lit_int = match &lit {
        Lit::Int(lit_int) => lit_int,
        _ => panic!("Literal must be an int"),
    };

    let instructions = generate_instructions(lit_int.to_string());

    TokenStream::from(quote! {
        #(#instructions)|*
    })
}

#[proc_macro]
pub fn opcode_list(tokens: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(tokens as Lit);
    let lit_int = match &lit {
        Lit::Int(lit_int) => lit_int,
        _ => panic!("Literal must be an int"),
    };

    let instructions = generate_instructions(lit_int.to_string());

    TokenStream::from(quote! {
        [#(#instructions),*]
    })
}

fn generate_instructions(bin_pat: String) -> Vec<u8> {
    let range = bin_pat
        .match_indices(|c| c == '_')
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let min_i = *range.iter().min().expect("Unabled to get min");
    let max_i = *range.iter().max().expect("Unable to get max");

    let step = 2u8.pow((bin_pat.len() - max_i - 1) as u32);
    let width = max_i - min_i + 1;
    let max_value = 2u8.pow(width as u32);

    let stripped_binary = bin_pat.replace("_", "0");
    let starting_val = u8::from_str_radix(&stripped_binary, 2).expect("Unable to parse as binary");

    (0..max_value).map(|i| starting_val + step * i).collect()
}
