//! Exemplos das lições 01 a 03 do Módulo 09: uma função que valida idades,
//! outra que panica de propósito, e os testes unitários das duas.

use std::num::ParseIntError;

/// Por que um texto não é uma idade válida.
///
/// Repare que o enum não deriva `PartialEq`: os testes usam `assert_matches!`.
#[derive(Debug)]
pub enum ErroIdade {
    /// O texto está vazio (ou só tem espaços).
    Vazia,
    /// O texto não é um número inteiro positivo.
    NaoNumero(ParseIntError),
    /// O número existe, mas passa de 150.
    ForaDaFaixa(u32),
}

/// Lê uma idade de 0 a 150, ignorando espaços nas pontas.
///
/// # Errors
///
/// Devolve [`ErroIdade`] se o texto estiver vazio, não for número ou passar de 150.
///
/// # Examples
///
/// ```
/// assert_eq!(idade::idade(" 42 ").unwrap(), 42);
/// assert!(idade::idade("abc").is_err());
/// ```
pub fn idade(texto: &str) -> Result<u32, ErroIdade> {
    let texto = texto.trim();
    if texto.is_empty() {
        return Err(ErroIdade::Vazia);
    }
    let n: u32 = texto.parse().map_err(ErroIdade::NaoNumero)?;
    if n > IDADE_MAXIMA {
        return Err(ErroIdade::ForaDaFaixa(n));
    }
    Ok(n)
}

const IDADE_MAXIMA: u32 = 150;

/// Quanto `parte` representa de `total`, em porcentagem inteira.
///
/// # Panics
///
/// Se `total` for zero.
pub fn porcentagem(parte: u32, total: u32) -> u32 {
    assert!(total != 0, "total não pode ser zero");
    parte * 100 / total
}

/// A média das idades válidas de uma lista; as inválidas são ignoradas.
pub fn media_idades(textos: &[&str]) -> Option<f64> {
    let validas: Vec<u32> = textos.iter().filter_map(|t| idade(t).ok()).collect();
    if validas.is_empty() {
        return None;
    }
    Some(soma(&validas) as f64 / validas.len() as f64)
}

/// Função privada: os testes unitários enxergam mesmo assim.
fn soma(valores: &[u32]) -> u32 {
    valores.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;
    use std::fs;
    use std::path::PathBuf;

    // ---------- Lição 01: um teste ignorado ----------

    #[test]
    #[ignore = "lento: roda só com --ignored"]
    fn todas_as_idades_validas() {
        for n in 0..=IDADE_MAXIMA {
            assert_eq!(idade(&n.to_string()).unwrap(), n);
        }
    }

    // ---------- Lição 02: testes unitários a fundo ----------

    #[test]
    fn idade_valida() {
        assert_eq!(idade("42").unwrap(), 42);
        assert_eq!(idade(" 7 ").unwrap(), 7);
    }

    #[test]
    fn soma_privada_e_testavel() {
        assert_eq!(soma(&[1, 2, 3]), 6);
    }

    #[test]
    #[should_panic(expected = "não pode ser zero")]
    fn porcentagem_de_zero_panica() {
        porcentagem(1, 0);
    }

    /// `?` no preparo; a verificação fica num `assert!`, que mostra a linha.
    #[test]
    fn media_usa_so_as_validas() -> Result<(), ErroIdade> {
        let base = idade("30")?;
        let media = media_idades(&["30", "abc", "50", ""]).expect("há duas idades válidas");
        assert!(
            (media - f64::from(base + 10)).abs() < 1e-9,
            "média foi {media}"
        );
        Ok(())
    }

    #[test]
    fn media_de_lista_sem_validas() {
        assert_eq!(media_idades(&["", "x"]), None);
    }

    /// *Fixture*: uma função que devolve o valor pronto.
    fn idades_de_exemplo() -> Vec<&'static str> {
        vec!["10", "20", "30", "999"]
    }

    #[test]
    fn media_da_fixture() {
        let media = media_idades(&idades_de_exemplo()).unwrap();
        assert!((media - 20.0).abs() < 1e-9);
    }

    /// Um arquivo temporário que se apaga sozinho no fim do teste (o `Drop`
    /// faz o papel do *teardown*).
    struct ArquivoTemporario {
        caminho: PathBuf,
    }

    impl ArquivoTemporario {
        fn novo(nome: &str, conteudo: &str) -> ArquivoTemporario {
            // Nome único por processo e por teste: os testes rodam em paralelo.
            let caminho =
                std::env::temp_dir().join(format!("m09-idade-{}-{nome}", std::process::id()));
            fs::write(&caminho, conteudo).unwrap();
            ArquivoTemporario { caminho }
        }
    }

    impl Drop for ArquivoTemporario {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.caminho);
        }
    }

    #[test]
    fn idades_lidas_de_arquivo() {
        let arquivo = ArquivoTemporario::novo("idades.txt", "18\n\n200\n40\n");
        let texto = fs::read_to_string(&arquivo.caminho).unwrap();
        let linhas: Vec<&str> = texto.lines().collect();
        assert_eq!(media_idades(&linhas), Some(29.0));
    } // `arquivo` sai de escopo aqui, e o Drop apaga o arquivo

    // ---------- Lição 03: assert_matches! ----------

    #[test]
    fn texto_vazio() {
        assert_matches!(idade("   "), Err(ErroIdade::Vazia));
    }

    #[test]
    fn texto_que_nao_e_numero() {
        assert_matches!(idade("abc"), Err(ErroIdade::NaoNumero(_)));
    }

    #[test]
    fn idade_acima_do_limite() {
        assert_matches!(idade("200"), Err(ErroIdade::ForaDaFaixa(n)) if n == 200);
    }

    #[test]
    fn negativo_nao_e_numero() {
        assert_matches!(
            idade("-1"),
            Err(ErroIdade::NaoNumero(_)),
            "u32 não aceita sinal de menos"
        );
    }
}
