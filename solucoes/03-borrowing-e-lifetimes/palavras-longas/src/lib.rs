/// Devolve as `n` palavras mais longas de `texto` (em número de `char`s),
/// da mais longa para a mais curta. Em caso de empate, vem primeiro a que
/// aparece antes no texto. Pontuação nas pontas da palavra é ignorada.
///
/// Nenhuma `String` é criada: cada `&str` devolvido aponta para dentro de `texto`.
pub fn palavras_mais_longas(texto: &str, n: usize) -> Vec<&str> {
    let mut palavras: Vec<&str> = texto
        .split_whitespace()
        .map(|p| p.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|p| !p.is_empty())
        .collect();
    // `sort_by_key` é estável: em empate, a ordem original se mantém.
    palavras.sort_by_key(|p| std::cmp::Reverse(p.chars().count()));
    palavras.truncate(n);
    palavras
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `p` está inteiramente dentro da memória de `texto`?
    fn aponta_para_dentro(texto: &str, p: &str) -> bool {
        let ini = texto.as_ptr() as usize;
        let fim = ini + texto.len();
        let q = p.as_ptr() as usize;
        q >= ini && q + p.len() <= fim
    }

    #[test]
    fn as_tres_maiores() {
        let texto = "O borrow checker verifica empréstimos em tempo de compilação.";
        assert_eq!(
            palavras_mais_longas(texto, 3),
            ["empréstimos", "compilação", "verifica"]
        );
    }

    #[test]
    fn empate_mantem_a_ordem_do_texto() {
        assert_eq!(palavras_mais_longas("ab cd ef g", 2), ["ab", "cd"]);
    }

    #[test]
    fn ignora_pontuacao_nas_pontas() {
        assert_eq!(palavras_mais_longas("(olá), mundo!", 2), ["mundo", "olá"]);
    }

    #[test]
    fn n_maior_que_o_total() {
        assert_eq!(palavras_mais_longas("um dois", 10), ["dois", "um"]);
        assert!(palavras_mais_longas("", 3).is_empty());
    }

    #[test]
    fn nao_aloca_strings_novas() {
        let texto = String::from("slices apontam para o texto original");
        for p in palavras_mais_longas(&texto, 4) {
            assert!(
                aponta_para_dentro(&texto, p),
                "{p:?} não aponta para o texto"
            );
        }
    }
}
