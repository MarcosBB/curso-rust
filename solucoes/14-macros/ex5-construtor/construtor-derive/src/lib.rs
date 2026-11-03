//! Exercício 5: `#[derive(Construtor)]` gera um `pub fn new(...) -> Self`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Gera `pub fn new(campo: Tipo, ...) -> Self`, com um parâmetro por campo.
/// Campos marcados com `#[construtor(padrao)]` recebem `Default::default()`.
#[proc_macro_derive(Construtor, attributes(construtor))]
pub fn derive_construtor(entrada: TokenStream) -> TokenStream {
    let entrada = parse_macro_input!(entrada as DeriveInput);
    expandir(entrada)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expandir(entrada: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let nome = &entrada.ident;
    let campos = match &entrada.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(c) => &c.named,
            outros => {
                return Err(syn::Error::new_spanned(
                    outros,
                    "`Construtor` precisa de campos nomeados",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                nome,
                "`Construtor` só pode ser derivado para structs",
            ));
        }
    };

    let mut parametros = Vec::new(); // `nome: Tipo`, na lista de argumentos
    let mut inicializacoes = Vec::new(); // `nome` ou `nome: Default::default()`
    for campo in campos {
        let ident = campo.ident.as_ref().expect("campo nomeado");
        let tipo = &campo.ty;

        let mut padrao = false;
        for attr in &campo.attrs {
            if attr.path().is_ident("construtor") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("padrao") {
                        padrao = true;
                        Ok(())
                    } else {
                        Err(meta.error("opção desconhecida; a única é `padrao`"))
                    }
                })?;
            }
        }

        if padrao {
            inicializacoes.push(quote! { #ident: ::std::default::Default::default() });
        } else {
            parametros.push(quote! { #ident: #tipo });
            inicializacoes.push(quote! { #ident });
        }
    }

    let (impl_generics, ty_generics, where_clause) = entrada.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #nome #ty_generics #where_clause {
            /// Cria o valor a partir de cada campo (gerado por `#[derive(Construtor)]`).
            pub fn new(#(#parametros),*) -> Self {
                Self { #(#inicializacoes),* }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn campo_padrao_nao_vira_parametro() {
        let entrada: DeriveInput = parse_quote! {
            struct C { a: i32, #[construtor(padrao)] b: Vec<u8> }
        };
        let gerado = expandir(entrada).unwrap().to_string();
        assert!(gerado.contains("pub fn new (a : i32)"), "{gerado}");
    }

    #[test]
    fn rejeita_enum_e_tupla() {
        let e: DeriveInput = parse_quote! { enum E { A } };
        assert!(expandir(e).is_err());
        let t: DeriveInput = parse_quote! { struct T(i32); };
        assert!(expandir(t).is_err());
    }
}
