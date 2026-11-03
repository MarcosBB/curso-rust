use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn expandir(texto: &LitStr) -> syn::Result<TokenStream> {
    let valor = texto.value();
    let erro = || syn::Error::new(texto.span(), "cor inválida: use o formato \"#rrggbb\"");

    let hex = valor.strip_prefix('#').ok_or_else(erro)?;
    if hex.len() != 6 {
        return Err(erro());
    }
    let canal = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| erro());
    let (r, g, b) = (canal(0)?, canal(2)?, canal(4)?);

    Ok(quote! { [#r, #g, #b] })
}
