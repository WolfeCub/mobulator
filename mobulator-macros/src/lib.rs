use proc_macro::TokenStream;

#[proc_macro]
pub fn gen_blah(tokens: TokenStream) -> TokenStream {
    let bin_pat = stringify!(expand!($pattern));

    let range = bin_pat.match_indices(|c| c == '_').map(|(i, _)| i).collect::<Vec<_>>();
    let min_i = *range.iter().min().expect("Unabled to get min");
    let max_i = *range.iter().max().expect("Unable to get max");
    let width = max_i - min_i + 1;

    let max_value = 2u8.pow(width as u32);

    let mut vec = Vec::with_capacity(usize::from(max_value));
    for num in 0..max_value {
        vec.push(format!("0b{}{:0width$b}{}", &bin_pat[..min_i], num, &bin_pat[max_i+1..],width = width));
    }

    vec.join("|");
    todo!()
}
