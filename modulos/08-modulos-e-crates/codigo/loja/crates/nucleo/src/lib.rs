//! O núcleo da loja: o estoque e as regras de venda (lição 07).

use std::collections::BTreeMap;
use thiserror::Error;

/// Erros das operações de estoque.
#[derive(Debug, Error, PartialEq)]
pub enum EstoqueError {
    /// O produto não está cadastrado.
    #[error("produto desconhecido: {0}")]
    Desconhecido(String),
    /// Não há unidades suficientes.
    #[error("estoque insuficiente de {produto}: pedido {pedido}, disponível {disponivel}")]
    Insuficiente {
        /// O produto pedido.
        produto: String,
        /// Quantas unidades foram pedidas.
        pedido: u32,
        /// Quantas unidades havia.
        disponivel: u32,
    },
}

/// Quantidade em estoque de cada produto, em ordem alfabética.
#[derive(Debug, Default)]
pub struct Estoque {
    itens: BTreeMap<String, u32>,
}

impl Estoque {
    /// Um estoque vazio.
    pub fn new() -> Estoque {
        Estoque::default()
    }

    /// Acrescenta `qtd` unidades de `produto`.
    pub fn repor(&mut self, produto: &str, qtd: u32) {
        *self.itens.entry(produto.to_string()).or_default() += qtd;
    }

    /// Quantas unidades de `produto` existem (0 se não cadastrado).
    pub fn quantidade(&self, produto: &str) -> u32 {
        self.itens.get(produto).copied().unwrap_or(0)
    }

    /// Retira `qtd` unidades de `produto`.
    pub fn vender(&mut self, produto: &str, qtd: u32) -> Result<(), EstoqueError> {
        let disponivel = self
            .itens
            .get_mut(produto)
            .ok_or_else(|| EstoqueError::Desconhecido(produto.to_string()))?;
        if *disponivel < qtd {
            return Err(EstoqueError::Insuficiente {
                produto: produto.to_string(),
                pedido: qtd,
                disponivel: *disponivel,
            });
        }
        *disponivel -= qtd;
        Ok(())
    }

    /// Uma linha por produto: `nome: quantidade`.
    #[cfg(feature = "relatorio")]
    pub fn relatorio(&self) -> String {
        self.itens
            .iter()
            .map(|(nome, qtd)| format!("{nome}: {qtd}\n"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vende_e_baixa_o_estoque() {
        let mut e = Estoque::new();
        e.repor("café", 3);
        assert_eq!(e.vender("café", 2), Ok(()));
        assert_eq!(e.quantidade("café"), 1);
    }

    #[test]
    fn recusa_venda_sem_estoque() {
        let mut e = Estoque::new();
        e.repor("pão", 1);
        assert_eq!(
            e.vender("pão", 5),
            Err(EstoqueError::Insuficiente {
                produto: "pão".to_string(),
                pedido: 5,
                disponivel: 1
            })
        );
        assert_eq!(
            e.vender("leite", 1),
            Err(EstoqueError::Desconhecido("leite".to_string()))
        );
    }

    #[cfg(feature = "relatorio")]
    #[test]
    fn relatorio_em_ordem_alfabetica() {
        let mut e = Estoque::new();
        e.repor("pão", 2);
        e.repor("café", 1);
        assert_eq!(e.relatorio(), "café: 1\npão: 2\n");
    }
}
