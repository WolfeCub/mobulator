use proc_macro::TokenStream;
use syn::{parse_macro_input, Lit};
use quote::quote;

#[proc_macro]
pub fn instructions(tokens: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(tokens as Lit);
    let lit_int = match &lit {
        Lit::Int(lit_int) => lit_int,
        _ => panic!("Literal must be an int"),
    };

    let bin_pat = lit_int.to_string();
    let range = bin_pat.match_indices(|c| c == '_').map(|(i, _)| i).collect::<Vec<_>>();
    let min_i = *range.iter().min().expect("Unabled to get min");
    let max_i = *range.iter().max().expect("Unable to get max");

    let step = 2u8.pow((bin_pat.len() - max_i - 1) as u32);
    let width = max_i - min_i + 1;
    let max_value = 2u8.pow(width as u32);

    let stripped_binary = lit_int.to_string().replace("_", "00");

    let mut instructions = Vec::with_capacity(usize::from(max_value));
    let mut permutation = u8::from_str_radix(&stripped_binary, 2).expect("Unable to parse as binary");
    for _ in 0..max_value {
        instructions.push(permutation);
        permutation += step;
    }

    TokenStream::from(quote! {
        #(#instructions)|*
    })
}
