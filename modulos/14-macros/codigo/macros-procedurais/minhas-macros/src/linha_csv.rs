use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr};

pub fn expandir(entrada: DeriveInput) -> syn::Result<TokenStream> {
    let nome = &entrada.ident;

    // 1. Só aceitamos structs com campos nomeados.
    let campos = match &entrada.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(campos) => &campos.named,
            outros => {
                return Err(syn::Error::new_spanned(
                    outros,
                    "`LinhaCsv` precisa de campos nomeados",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                nome,
                "`LinhaCsv` só pode ser derivada para structs",
            ));
        }
    };

    // 2. Para cada campo, o título da coluna e o nome do campo.
    let mut titulos = Vec::new();
    let mut valores = Vec::new();
    for campo in campos {
        let ident = campo.ident.as_ref().expect("campo nomeado tem nome");
        let mut titulo = ident.to_string();
        let mut pular = false;

        for attr in &campo.attrs {
            if !attr.path().is_ident("csv") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("renomear") {
                    let valor: LitStr = meta.value()?.parse()?;
                    titulo = valor.value();
                    Ok(())
                } else if meta.path.is_ident("pular") {
                    pular = true;
                    Ok(())
                } else {
                    Err(meta.error("opção desconhecida; use `renomear = \"...\"` ou `pular`"))
                }
            })?;
        }

        if !pular {
            titulos.push(titulo);
            // `quote_spanned!` marca o código com a posição do tipo do campo:
            // se o tipo não tiver `Display`, o erro aponta para o campo.
            valores.push(quote_spanned! {campo.ty.span()=>
                ::std::string::ToString::to_string(&self.#ident)
            });
        }
    }

    // 3. O código gerado. Caminhos absolutos (`::std`, `::linha_csv`) para não
    //    depender do que o usuário importou.
    let (impl_generics, ty_generics, where_clause) = entrada.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::linha_csv::LinhaCsv for #nome #ty_generics #where_clause {
            fn cabecalho() -> ::std::vec::Vec<&'static str> {
                ::std::vec![#(#titulos),*]
            }

            fn linha(&self) -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(#valores),*]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn gera_o_cabecalho_com_renomear_e_pular() {
        let entrada: DeriveInput = parse_quote! {
            struct Pessoa {
                #[csv(renomear = "nome completo")]
                nome: String,
                idade: u32,
                #[csv(pular)]
                senha: String,
            }
        };
        let gerado = expandir(entrada).unwrap().to_string();
        assert!(
            gerado.contains(r#"vec ! ["nome completo" , "idade"]"#),
            "{gerado}"
        );
        assert!(!gerado.contains("senha"), "{gerado}");
    }

    #[test]
    fn rejeita_enum() {
        let entrada: DeriveInput = parse_quote! {
            enum Cor { Vermelho, Verde }
        };
        let erro = expandir(entrada).unwrap_err();
        assert_eq!(
            erro.to_string(),
            "`LinhaCsv` só pode ser derivada para structs"
        );
    }

    #[test]
    fn rejeita_opcao_desconhecida() {
        let entrada: DeriveInput = parse_quote! {
            struct P { #[csv(renomar = "x")] a: i32 }
        };
        let erro = expandir(entrada).unwrap_err();
        assert!(erro.to_string().starts_with("opção desconhecida"));
    }
}
