use std::thread;

/// Tamanho de cada pedaço para dividir `n` itens em `partes` (no mínimo 1).
fn tamanho_do_pedaco(n: usize, partes: usize) -> usize {
    n.div_ceil(partes.max(1)).max(1)
}

/// Soma `dados` com até `partes` threads.
pub fn soma_paralela(dados: &[u64], partes: usize) -> u64 {
    let tamanho = tamanho_do_pedaco(dados.len(), partes);
    thread::scope(|s| {
        let mut handles = Vec::new();
        for pedaco in dados.chunks(tamanho) {
            handles.push(s.spawn(move || pedaco.iter().sum::<u64>()));
        }
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

/// O maior valor de `dados`, com até `partes` threads (`None` se vazio).
pub fn maximo_paralelo(dados: &[i64], partes: usize) -> Option<i64> {
    let tamanho = tamanho_do_pedaco(dados.len(), partes);
    thread::scope(|s| {
        let handles: Vec<_> = dados
            .chunks(tamanho)
            .map(|pedaco| s.spawn(move || pedaco.iter().copied().max()))
            .collect();
        handles.into_iter().filter_map(|h| h.join().unwrap()).max()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_igual_a_sequencial() {
        let dados: Vec<u64> = (1..=10_000).collect();
        for partes in [1, 2, 3, 8, 100] {
            assert_eq!(soma_paralela(&dados, partes), 50_005_000);
        }
    }

    #[test]
    fn casos_de_borda() {
        assert_eq!(soma_paralela(&[], 4), 0);
        assert_eq!(soma_paralela(&[7], 0), 7);
        assert_eq!(maximo_paralelo(&[], 4), None);
    }

    #[test]
    fn maximo() {
        let dados = [3, -1, 42, 7, 42, -100, 0];
        for partes in 1..=8 {
            assert_eq!(maximo_paralelo(&dados, partes), Some(42));
        }
    }

    #[test]
    fn dados_continuam_disponiveis() {
        let dados = vec![1, 2, 3];
        let s = soma_paralela(&dados, 2);
        assert_eq!(s, dados.iter().sum::<u64>());
    }
}
