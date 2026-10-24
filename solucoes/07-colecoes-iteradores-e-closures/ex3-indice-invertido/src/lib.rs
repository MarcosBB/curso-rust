use std::collections::{BTreeMap, BTreeSet};

/// Palavra (em minúsculas) → números das linhas (a partir de 1) em que aparece.
/// A pontuação nas pontas da palavra é descartada.
pub fn indice(texto: &str) -> BTreeMap<String, BTreeSet<usize>> {
    let mut idx: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (i, linha) in texto.lines().enumerate() {
        for bruta in linha.split_whitespace() {
            let palavra = bruta
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if palavra.is_empty() {
                continue;
            }
            idx.entry(palavra).or_default().insert(i + 1);
        }
    }
    idx
}

/// Linhas que contêm **todas** as palavras pedidas.
pub fn linhas_com_todas(
    idx: &BTreeMap<String, BTreeSet<usize>>,
    palavras: &[&str],
) -> BTreeSet<usize> {
    let mut conjuntos = palavras.iter().map(|p| idx.get(&p.to_lowercase()));
    // Sem palavras, nenhuma linha. Uma palavra ausente também zera o resultado.
    let Some(Some(primeiro)) = conjuntos.next() else {
        return BTreeSet::new();
    };
    let mut resultado = primeiro.clone();
    for c in conjuntos {
        match c {
            Some(c) => resultado.retain(|linha| c.contains(linha)),
            None => return BTreeSet::new(),
        }
    }
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXTO: &str = "\
O rato roeu a roupa.
A roupa era do Rei!
O rei, de Roma, riu do rato.";

    fn linhas(v: &[usize]) -> BTreeSet<usize> {
        v.iter().copied().collect()
    }

    #[test]
    fn monta_o_indice() {
        let idx = indice(TEXTO);
        assert_eq!(idx["rato"], linhas(&[1, 3]));
        assert_eq!(idx["rei"], linhas(&[2, 3]));
        assert_eq!(idx["roupa"], linhas(&[1, 2]));
        assert!(!idx.contains_key("Rei"));
        assert!(!idx.contains_key("rei!"));
    }

    #[test]
    fn chaves_em_ordem() {
        let idx = indice("b a\nc a");
        let chaves: Vec<&str> = idx.keys().map(|k| k.as_str()).collect();
        assert_eq!(chaves, ["a", "b", "c"]);
        assert_eq!(idx["a"], linhas(&[1, 2]));
    }

    #[test]
    fn ignora_pontuacao_solta() {
        let idx = indice("oi - tchau");
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn busca_com_todas() {
        let idx = indice(TEXTO);
        assert_eq!(linhas_com_todas(&idx, &["rato", "Rei"]), linhas(&[3]));
        assert_eq!(linhas_com_todas(&idx, &["roupa"]), linhas(&[1, 2]));
        assert!(linhas_com_todas(&idx, &["rato", "dragão"]).is_empty());
        assert!(linhas_com_todas(&idx, &[]).is_empty());
    }
}
