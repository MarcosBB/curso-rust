//! Macros procedurais do Módulo 14: um derive, um atributo e uma function-like.
//!
//! Cada função pública só converte a entrada e chama a função `expandir` do
//! módulo correspondente, que trabalha com `proc_macro2` e pode ser testada.

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, LitStr, parse_macro_input};

mod cor;
mod cronometrar;
mod linha_csv;

/// Gera `impl linha_csv::LinhaCsv` para uma struct com campos nomeados.
///
/// Atributos auxiliares nos campos: `#[csv(renomear = "titulo")]` e `#[csv(pular)]`.
#[proc_macro_derive(LinhaCsv, attributes(csv))]
pub fn derive_linha_csv(entrada: TokenStream) -> TokenStream {
    let entrada = parse_macro_input!(entrada as DeriveInput);
    linha_csv::expandir(entrada)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Mede o tempo de uma função e imprime no stderr ao terminar.
#[proc_macro_attribute]
pub fn cronometrar(args: TokenStream, item: TokenStream) -> TokenStream {
    let funcao = parse_macro_input!(item as ItemFn);
    cronometrar::expandir(args.into(), funcao)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Converte `"#rrggbb"` em `[u8; 3]` durante a compilação.
#[proc_macro]
pub fn cor(entrada: TokenStream) -> TokenStream {
    let texto = parse_macro_input!(entrada as LitStr);
    cor::expandir(&texto)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
