use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemFn};

pub fn expandir(args: TokenStream, funcao: ItemFn) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
            "`cronometrar` não recebe argumentos",
        ));
    }
    if let Some(token) = &funcao.sig.asyncness {
        return Err(syn::Error::new_spanned(
            token,
            "`cronometrar` não funciona com `async fn`",
        ));
    }

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = funcao;
    let nome = sig.ident.to_string();

    // Nomes com `Span::mixed_site()` se comportam como os de `macro_rules!`:
    // não colidem com um parâmetro do usuário que se chame `inicio`.
    let inicio = Ident::new("inicio", Span::mixed_site());
    let resultado = Ident::new("resultado", Span::mixed_site());

    // O corpo original vira uma closure chamada na hora: um `return` dentro
    // dele sai da closure, e o tempo é impresso do mesmo jeito.
    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            let #inicio = ::std::time::Instant::now();
            let #resultado = (|| #block)();
            ::std::eprintln!("[cronometrar] {} levou {:?}", #nome, #inicio.elapsed());
            #resultado
        }
    })
}
