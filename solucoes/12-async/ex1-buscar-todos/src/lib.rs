use tokio::time::{Duration, sleep};

/// Simula uma consulta pela rede: demora 100 ms.
pub async fn buscar(id: u32) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("usuario-{id}")
}

/// Busca todos os ids ao mesmo tempo e devolve na ordem dos ids.
pub async fn buscar_todos(ids: &[u32]) -> Vec<String> {
    let handles: Vec<_> = ids.iter().map(|&id| tokio::spawn(buscar(id))).collect();
    let mut saida = Vec::with_capacity(handles.len());
    for h in handles {
        saida.push(h.await.expect("buscar não dá panic"));
    }
    saida
}

/// Busca as duas listas ao mesmo tempo.
pub async fn buscar_dois_grupos(a: &[u32], b: &[u32]) -> (Vec<String>, Vec<String>) {
    tokio::join!(buscar_todos(a), buscar_todos(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    // `start_paused`: o relógio do tokio fica parado e avança sozinho quando
    // todas as tasks estão esperando um timer. O teste roda em microssegundos
    // e o tempo medido é exato.
    #[tokio::test(start_paused = true)]
    async fn devolve_na_ordem() {
        let r = buscar_todos(&[3, 1, 2]).await;
        assert_eq!(r, ["usuario-3", "usuario-1", "usuario-2"]);
    }

    #[tokio::test(start_paused = true)]
    async fn roda_ao_mesmo_tempo() {
        let inicio = Instant::now();
        let ids: Vec<u32> = (0..50).collect();
        assert_eq!(buscar_todos(&ids).await.len(), 50);
        assert!(
            inicio.elapsed() < Duration::from_millis(150),
            "{:?}",
            inicio.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dois_grupos_ao_mesmo_tempo() {
        let inicio = Instant::now();
        let (a, b) = buscar_dois_grupos(&[1, 2], &[3]).await;
        assert_eq!(a, ["usuario-1", "usuario-2"]);
        assert_eq!(b, ["usuario-3"]);
        assert!(
            inicio.elapsed() < Duration::from_millis(150),
            "{:?}",
            inicio.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn vazio() {
        assert!(buscar_todos(&[]).await.is_empty());
    }
}
