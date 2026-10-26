//! Exemplo de *features* (lição 06): o núcleo sempre existe, e cada feature
//! acrescenta funções.
//!
//! | Feature     | Padrão | O que acrescenta                  |
//! |-------------|--------|-----------------------------------|
//! | `moeda`     | sim    | `formatar`                        |
//! | `relatorio` | não    | `relatorio` (liga `moeda`)        |
//! | `json`      | não    | `para_json` (usa `serde_json`)    |

/// Uma venda: o produto e o valor em centavos.
#[derive(Debug, Clone, PartialEq)]
pub struct Venda {
    /// Nome do produto.
    pub produto: String,
    /// Valor em centavos.
    pub centavos: u64,
}

impl Venda {
    /// Cria uma venda.
    pub fn nova(produto: &str, centavos: u64) -> Venda {
        Venda {
            produto: produto.to_string(),
            centavos,
        }
    }
}

/// Soma das vendas, em centavos. Existe com qualquer combinação de features.
pub fn total(vendas: &[Venda]) -> u64 {
    vendas.iter().map(|v| v.centavos).sum()
}

/// Formata centavos como `R$ 12,50`.
#[cfg(feature = "moeda")]
pub fn formatar(centavos: u64) -> String {
    format!("R$ {},{:02}", centavos / 100, centavos % 100)
}

/// Relatório em texto, uma linha por venda e o total no fim.
#[cfg(feature = "relatorio")]
pub fn relatorio(vendas: &[Venda]) -> String {
    let mut s = String::new();
    for v in vendas {
        s.push_str(&format!("{:<10} {:>10}\n", v.produto, formatar(v.centavos)));
    }
    s.push_str(&format!(
        "{:<10} {:>10}\n",
        "TOTAL",
        formatar(total(vendas))
    ));
    s
}

/// As vendas como um array JSON.
#[cfg(feature = "json")]
pub fn para_json(vendas: &[Venda]) -> String {
    let itens: Vec<serde_json::Value> = vendas
        .iter()
        .map(|v| serde_json::json!({ "produto": v.produto, "centavos": v.centavos }))
        .collect();
    serde_json::Value::Array(itens).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exemplo() -> Vec<Venda> {
        vec![Venda::nova("café", 1250), Venda::nova("pão", 375)]
    }

    #[test]
    fn total_soma_os_centavos() {
        assert_eq!(total(&exemplo()), 1625);
    }

    #[cfg(feature = "moeda")]
    #[test]
    fn formata_em_reais() {
        assert_eq!(formatar(1625), "R$ 16,25");
        assert_eq!(formatar(5), "R$ 0,05");
    }

    #[cfg(feature = "relatorio")]
    #[test]
    fn relatorio_termina_com_o_total() {
        let r = relatorio(&exemplo());
        assert!(r.ends_with("TOTAL        R$ 16,25\n"), "{r}");
    }

    #[cfg(feature = "json")]
    #[test]
    fn exporta_json() {
        assert_eq!(
            para_json(&exemplo()[..1]),
            r#"[{"centavos":1250,"produto":"café"}]"#
        );
    }
}
