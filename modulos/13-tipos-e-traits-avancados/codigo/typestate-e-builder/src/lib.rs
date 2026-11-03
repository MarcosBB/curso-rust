//! Lições 03 e 04: `PhantomData`, typestate e builders.
//!
//! Os blocos marcados com `compile_fail` abaixo são *doctests* que passam
//! quando o código **não** compila. É assim que se testa que um uso errado
//! da API é recusado pelo compilador.

pub mod ids {
    use std::fmt;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;

    /// Um id que lembra a que entidade pertence.
    ///
    /// `Id<Usuario>` e `Id<Pedido>` são tipos diferentes, embora os dois sejam só um `u64`:
    ///
    /// ```compile_fail,E0308
    /// use typestate_e_builder::ids::Id;
    /// struct Usuario;
    /// struct Pedido;
    /// fn buscar_pedido(_: Id<Pedido>) {}
    /// let u: Id<Usuario> = Id::new(7);
    /// buscar_pedido(u); // erro: esperava Id<Pedido>
    /// ```
    pub struct Id<T> {
        valor: u64,
        // `fn() -> T`: o id não é dono de um `T` (veja a lição 07).
        _marca: PhantomData<fn() -> T>,
    }

    impl<T> Id<T> {
        pub fn new(valor: u64) -> Self {
            Id {
                valor,
                _marca: PhantomData,
            }
        }

        pub fn valor(self) -> u64 {
            self.valor
        }
    }

    // Implementações à mão: o `derive` exigiria `T: Clone`, `T: PartialEq`...
    impl<T> Clone for Id<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T> Copy for Id<T> {}

    impl<T> PartialEq for Id<T> {
        fn eq(&self, outro: &Self) -> bool {
            self.valor == outro.valor
        }
    }

    impl<T> Eq for Id<T> {}

    impl<T> Hash for Id<T> {
        fn hash<H: Hasher>(&self, h: &mut H) {
            self.valor.hash(h);
        }
    }

    impl<T> fmt::Debug for Id<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let nome = std::any::type_name::<T>()
                .rsplit("::")
                .next()
                .unwrap_or("?");
            write!(f, "Id<{nome}>({})", self.valor)
        }
    }
}

pub mod pedido {
    use std::marker::PhantomData;

    // Os estados: tipos vazios, que só existem na compilação.
    #[derive(Debug)]
    pub struct Carrinho;
    #[derive(Debug)]
    pub struct Pago;
    #[derive(Debug)]
    pub struct Enviado;

    /// Algo que todo estado sabe dizer.
    pub trait Estado {
        const NOME: &'static str;
    }

    impl Estado for Carrinho {
        const NOME: &'static str = "carrinho";
    }
    impl Estado for Pago {
        const NOME: &'static str = "pago";
    }
    impl Estado for Enviado {
        const NOME: &'static str = "enviado";
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Item {
        pub nome: String,
        pub preco_centavos: u64,
    }

    /// Um pedido cujo estado faz parte do tipo.
    ///
    /// Enviar um pedido que ainda é carrinho não compila:
    ///
    /// ```compile_fail,E0599
    /// use typestate_e_builder::pedido::Pedido;
    /// let p = Pedido::new();
    /// p.enviar("BR123"); // erro: `enviar` não existe em `Pedido<Carrinho>`
    /// ```
    ///
    /// Nem pagar duas vezes (a transição consome o pedido):
    ///
    /// ```compile_fail,E0382
    /// use typestate_e_builder::pedido::Pedido;
    /// let p = Pedido::new();
    /// let pago = p.pagar().unwrap();
    /// let de_novo = p.pagar(); // erro: `p` foi movido
    /// ```
    #[derive(Debug)]
    pub struct Pedido<E> {
        itens: Vec<Item>,
        total_centavos: u64,
        rastreio: Option<String>,
        _estado: PhantomData<E>,
    }

    impl Default for Pedido<Carrinho> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Pedido<Carrinho> {
        pub fn new() -> Self {
            Pedido {
                itens: Vec::new(),
                total_centavos: 0,
                rastreio: None,
                _estado: PhantomData,
            }
        }

        pub fn adicionar(&mut self, nome: &str, preco_centavos: u64) {
            self.itens.push(Item {
                nome: nome.to_string(),
                preco_centavos,
            });
        }

        /// Carrinho vazio não pode ser pago: devolve o carrinho de volta no `Err`.
        pub fn pagar(self) -> Result<Pedido<Pago>, Pedido<Carrinho>> {
            if self.itens.is_empty() {
                return Err(self);
            }
            let total = self.itens.iter().map(|i| i.preco_centavos).sum();
            Ok(Pedido {
                itens: self.itens,
                total_centavos: total,
                rastreio: None,
                _estado: PhantomData,
            })
        }
    }

    impl Pedido<Pago> {
        pub fn total_centavos(&self) -> u64 {
            self.total_centavos
        }

        pub fn enviar(self, rastreio: &str) -> Pedido<Enviado> {
            Pedido {
                itens: self.itens,
                total_centavos: self.total_centavos,
                rastreio: Some(rastreio.to_string()),
                _estado: PhantomData,
            }
        }
    }

    impl Pedido<Enviado> {
        pub fn rastreio(&self) -> &str {
            self.rastreio.as_deref().unwrap_or_default()
        }
    }

    // Métodos que valem em qualquer estado.
    impl<E: Estado> Pedido<E> {
        pub fn itens(&self) -> &[Item] {
            &self.itens
        }

        pub fn status(&self) -> &'static str {
            E::NOME
        }
    }
}

pub mod requisicao {
    use std::marker::PhantomData;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Requisicao {
        pub url: String,
        pub metodo: &'static str,
        pub cabecalhos: Vec<(String, String)>,
        pub tempo_limite: Duration,
    }

    impl Requisicao {
        pub fn get() -> RequisicaoBuilder<SemUrl> {
            RequisicaoBuilder::new("GET")
        }

        pub fn post() -> RequisicaoBuilder<SemUrl> {
            RequisicaoBuilder::new("POST")
        }
    }

    pub struct SemUrl;
    pub struct ComUrl;

    /// Builder com o campo obrigatório (a URL) verificado na compilação.
    ///
    /// ```compile_fail,E0599
    /// use typestate_e_builder::requisicao::Requisicao;
    /// let r = Requisicao::get().cabecalho("Accept", "text/html").build(); // sem URL
    /// ```
    #[must_use = "um builder não faz nada até `build()`"]
    pub struct RequisicaoBuilder<U> {
        url: Option<String>,
        metodo: &'static str,
        cabecalhos: Vec<(String, String)>,
        tempo_limite: Duration,
        _u: PhantomData<U>,
    }

    impl RequisicaoBuilder<SemUrl> {
        fn new(metodo: &'static str) -> Self {
            RequisicaoBuilder {
                url: None,
                metodo,
                cabecalhos: Vec::new(),
                tempo_limite: Duration::from_secs(30),
                _u: PhantomData,
            }
        }

        pub fn url(self, url: impl Into<String>) -> RequisicaoBuilder<ComUrl> {
            RequisicaoBuilder {
                url: Some(url.into()),
                metodo: self.metodo,
                cabecalhos: self.cabecalhos,
                tempo_limite: self.tempo_limite,
                _u: PhantomData,
            }
        }
    }

    // Opcionais: valem antes e depois da URL.
    impl<U> RequisicaoBuilder<U> {
        pub fn cabecalho(mut self, nome: &str, valor: &str) -> Self {
            self.cabecalhos.push((nome.to_string(), valor.to_string()));
            self
        }

        pub fn tempo_limite(mut self, t: Duration) -> Self {
            self.tempo_limite = t;
            self
        }
    }

    impl RequisicaoBuilder<ComUrl> {
        pub fn build(self) -> Requisicao {
            Requisicao {
                url: self.url.expect("o estado ComUrl garante a URL"),
                metodo: self.metodo,
                cabecalhos: self.cabecalhos,
                tempo_limite: self.tempo_limite,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ids::Id;
    use super::pedido::{Pago, Pedido};
    use super::requisicao::Requisicao;
    use std::collections::HashSet;
    use std::time::Duration;

    struct Usuario; // não é Clone, nem PartialEq, nem Hash

    #[test]
    fn id_e_copy_mesmo_sem_t_ser() {
        let a: Id<Usuario> = Id::new(7);
        let b = a;
        assert_eq!(a, b);
        let conjunto: HashSet<Id<Usuario>> = [a, b, Id::new(8)].into_iter().collect();
        assert_eq!(conjunto.len(), 2);
        assert_eq!(format!("{a:?}"), "Id<Usuario>(7)");
    }

    #[test]
    fn fluxo_do_pedido() {
        let mut p = Pedido::new();
        assert_eq!(p.status(), "carrinho");
        p.adicionar("livro", 4_990);
        p.adicionar("caneta", 350);
        let pago: Pedido<Pago> = p.pagar().expect("tem itens");
        assert_eq!(pago.status(), "pago");
        assert_eq!(pago.total_centavos(), 5_340);
        let enviado = pago.enviar("BR123");
        assert_eq!(enviado.status(), "enviado");
        assert_eq!(enviado.rastreio(), "BR123");
        assert_eq!(enviado.itens().len(), 2);
    }

    #[test]
    fn carrinho_vazio_volta_no_err() {
        let vazio = Pedido::new();
        let de_volta = vazio.pagar().unwrap_err();
        assert_eq!(de_volta.status(), "carrinho");
    }

    #[test]
    fn builder_monta_a_requisicao() {
        let r = Requisicao::post()
            .cabecalho("Accept", "application/json")
            .url("https://exemplo.com/api")
            .tempo_limite(Duration::from_secs(5))
            .build();
        assert_eq!(r.metodo, "POST");
        assert_eq!(r.url, "https://exemplo.com/api");
        assert_eq!(r.cabecalhos.len(), 1);
        assert_eq!(r.tempo_limite, Duration::from_secs(5));
    }

    #[test]
    fn builder_usa_padroes() {
        let r = Requisicao::get().url("https://exemplo.com").build();
        assert_eq!(r.tempo_limite, Duration::from_secs(30));
        assert!(r.cabecalhos.is_empty());
    }
}
