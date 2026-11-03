use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

/// Conta as palavras (em minúsculas) de cada texto, uma contagem por texto.
fn contar(texto: &str) -> HashMap<String, usize> {
    let mut mapa = HashMap::new();
    for p in texto.split_whitespace() {
        *mapa.entry(p.to_lowercase()).or_insert(0) += 1;
    }
    mapa
}

/// Distribui `textos` entre `workers` threads. Cada worker conta o seu lote
/// e manda um mapa parcial pelo canal; quem chamou junta os mapas.
pub fn contar_palavras(textos: Vec<String>, workers: usize) -> HashMap<String, usize> {
    let workers = workers.max(1);
    let mut lotes: Vec<Vec<String>> = vec![Vec::new(); workers];
    for (i, t) in textos.into_iter().enumerate() {
        lotes[i % workers].push(t);
    }

    let (tx, rx) = mpsc::channel();
    for lote in lotes {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut parcial = HashMap::new();
            for texto in &lote {
                for (p, n) in contar(texto) {
                    *parcial.entry(p).or_insert(0) += n;
                }
            }
            tx.send(parcial).unwrap();
        });
    }
    drop(tx);

    let mut total = HashMap::new();
    for parcial in rx {
        for (p, n) in parcial {
            *total.entry(p).or_insert(0) += n;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textos() -> Vec<String> {
        [
            "O rato roeu",
            "a roupa do rei",
            "de Roma o rato",
            "",
            "rei rei",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    #[test]
    fn igual_a_sequencial() {
        let esperado = contar(&textos().join(" "));
        for w in [1, 2, 3, 10] {
            assert_eq!(contar_palavras(textos(), w), esperado);
        }
    }

    #[test]
    fn contagens() {
        let c = contar_palavras(textos(), 3);
        assert_eq!(c["rato"], 2);
        assert_eq!(c["rei"], 3);
        assert_eq!(c["o"], 2);
        assert_eq!(c.get("gato"), None);
    }

    #[test]
    fn sem_textos_e_zero_workers() {
        assert!(contar_palavras(Vec::new(), 4).is_empty());
        assert_eq!(contar_palavras(vec!["a a".into()], 0)["a"], 2);
    }
}
