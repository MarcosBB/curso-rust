/// Adiciona `novo` ao fim de `nomes` e devolve o tamanho, em bytes,
/// do maior nome que existia ANTES da inserção (0 se não havia nenhum).
pub fn registra(nomes: &mut Vec<String>, novo: &str) -> usize {
    // Guarda só o número (um `usize` é Copy), e não a referência ao nome.
    let maior = nomes.iter().map(|n| n.len()).max().unwrap_or(0);
    nomes.push(novo.to_string());
    maior
}

/// Acrescenta "fim" a `itens` e devolve um resumo com o primeiro item
/// e o total DEPOIS da inserção.
pub fn resumo(itens: &mut Vec<String>) -> String {
    // Muta primeiro e só depois pega um empréstimo novo para ler.
    itens.push(String::from("fim"));
    format!("primeiro: {}, total: {}", itens[0], itens.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registra_devolve_o_maior_anterior() {
        let mut v = vec![String::from("Ana"), String::from("Bartolomeu")];
        assert_eq!(registra(&mut v, "Maximiliano Augusto"), 10);
        assert_eq!(v, ["Ana", "Bartolomeu", "Maximiliano Augusto"]);
    }

    #[test]
    fn registra_com_vetor_vazio() {
        let mut v = Vec::new();
        assert_eq!(registra(&mut v, "Zé"), 0);
        assert_eq!(v, ["Zé"]);
    }

    #[test]
    fn resumo_conta_depois_do_push() {
        let mut v = vec![String::from("a"), String::from("b")];
        assert_eq!(resumo(&mut v), "primeiro: a, total: 3");
        assert_eq!(v, ["a", "b", "fim"]);
    }
}
