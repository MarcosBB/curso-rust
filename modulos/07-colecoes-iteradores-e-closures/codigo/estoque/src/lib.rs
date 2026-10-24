//! Um estoque pequeno que usa as coleções das lições 01 a 03:
//! `HashMap` com a Entry API, `retain`, `BTreeMap` para relatórios
//! ordenados, `HashSet` para categorias e `BinaryHeap` com `Reverse`
//! para decidir o que repor primeiro.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub categoria: String,
    pub quantidade: u32,
}

#[derive(Debug, Default)]
pub struct Estoque {
    itens: HashMap<String, Item>,
}

impl Estoque {
    pub fn new() -> Self {
        Self::default()
    }

    /// Soma `qtd` ao item, criando-o se ainda não existir (lição 02).
    pub fn entrada(&mut self, nome: &str, categoria: &str, qtd: u32) {
        self.itens
            .entry(nome.to_string())
            .and_modify(|item| item.quantidade += qtd)
            .or_insert_with(|| Item {
                categoria: categoria.to_string(),
                quantidade: qtd,
            });
    }

    /// Tira `qtd` do item. Devolve o que sobrou, ou `Err` se não há o bastante.
    pub fn saida(&mut self, nome: &str, qtd: u32) -> Result<u32, String> {
        // `get_mut` aceita `&str` num mapa de `String` (Borrow).
        let item = self
            .itens
            .get_mut(nome)
            .ok_or_else(|| format!("{nome}: item desconhecido"))?;
        if item.quantidade < qtd {
            return Err(format!("{nome}: só há {}", item.quantidade));
        }
        item.quantidade -= qtd;
        Ok(item.quantidade)
    }

    pub fn quantidade(&self, nome: &str) -> Option<u32> {
        self.itens.get(nome).map(|item| item.quantidade)
    }

    /// Remove os itens zerados e devolve quantos saíram (`retain`).
    pub fn limpar_zerados(&mut self) -> usize {
        let antes = self.itens.len();
        self.itens.retain(|_, item| item.quantidade > 0);
        antes - self.itens.len()
    }

    /// Total por categoria, em ordem alfabética (`BTreeMap`).
    pub fn total_por_categoria(&self) -> BTreeMap<&str, u32> {
        let mut totais = BTreeMap::new();
        for item in self.itens.values() {
            *totais.entry(item.categoria.as_str()).or_insert(0) += item.quantidade;
        }
        totais
    }

    /// Categorias distintas (`HashSet`).
    pub fn categorias(&self) -> HashSet<&str> {
        self.itens.values().map(|i| i.categoria.as_str()).collect()
    }

    /// Os `n` itens com menos unidades, do menor para o maior.
    /// `BinaryHeap` é max-heap; `Reverse` o transforma em min-heap.
    /// O nome entra na tupla para desempatar de forma determinística.
    pub fn repor_primeiro(&self, n: usize) -> Vec<(&str, u32)> {
        let mut heap = BinaryHeap::new();
        for (nome, item) in &self.itens {
            heap.push(Reverse((item.quantidade, nome.as_str())));
        }
        let mut saida = Vec::new();
        while saida.len() < n {
            let Some(Reverse((qtd, nome))) = heap.pop() else {
                break;
            };
            saida.push((nome, qtd));
        }
        saida
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exemplo() -> Estoque {
        let mut e = Estoque::new();
        e.entrada("caneta", "papelaria", 10);
        e.entrada("caderno", "papelaria", 3);
        e.entrada("cabo usb", "eletrônicos", 5);
        e.entrada("caneta", "papelaria", 2);
        e
    }

    #[test]
    fn entrada_soma_na_mesma_chave() {
        assert_eq!(exemplo().quantidade("caneta"), Some(12));
    }

    #[test]
    fn saida_valida_quantidade() {
        let mut e = exemplo();
        assert_eq!(e.saida("caderno", 2), Ok(1));
        assert_eq!(e.saida("caderno", 5), Err("caderno: só há 1".to_string()));
        assert!(e.saida("lápis", 1).is_err());
    }

    #[test]
    fn limpar_zerados() {
        let mut e = exemplo();
        e.saida("caderno", 3).unwrap();
        assert_eq!(e.limpar_zerados(), 1);
        assert_eq!(e.quantidade("caderno"), None);
    }

    #[test]
    fn totais_ordenados() {
        let e = exemplo();
        let totais: Vec<_> = e.total_por_categoria().into_iter().collect();
        assert_eq!(totais, [("eletrônicos", 5), ("papelaria", 15)]);
    }

    #[test]
    fn categorias_unicas() {
        let e = exemplo();
        let cats = e.categorias();
        assert_eq!(cats.len(), 2);
        assert!(cats.contains("papelaria"));
    }

    #[test]
    fn repor_os_menores_primeiro() {
        let e = exemplo();
        assert_eq!(e.repor_primeiro(2), [("caderno", 3), ("cabo usb", 5)]);
        assert_eq!(e.repor_primeiro(10).len(), 3);
    }
}
