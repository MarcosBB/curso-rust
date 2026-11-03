use std::sync::{Arc, Mutex};
use tokio::time::{Duration, sleep};

async fn salvar(log: Arc<Mutex<Vec<String>>>, item: String) {
    log.lock().unwrap().push(format!("começando {item}"));
    sleep(Duration::from_millis(10)).await; // simula E/S
    log.lock().unwrap().push(format!("salvo {item}"));
}

/// Salva todos os itens ao mesmo tempo e devolve o log.
pub async fn salvar_todos(itens: Vec<String>) -> Vec<String> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let prefixo = Arc::new(String::from("item-"));
    let mut handles = Vec::new();
    for item in itens {
        let nome = format!("{prefixo}{item}");
        handles.push(tokio::spawn(salvar(log.clone(), nome)));
    }
    for h in handles {
        h.await.unwrap();
    }
    log.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test(start_paused = true)]
    async fn registra_tudo() {
        let itens: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        let log = tokio::spawn(salvar_todos(itens)).await.unwrap();
        assert_eq!(log.len(), 6);
        for x in ["a", "b", "c"] {
            assert!(log.contains(&format!("começando item-{x}")));
            assert!(log.contains(&format!("salvo item-{x}")));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn nao_serializa() {
        let itens: Vec<String> = (0..10).map(|i| i.to_string()).collect();
        let inicio = Instant::now();
        tokio::spawn(salvar_todos(itens)).await.unwrap();
        assert!(
            inicio.elapsed() < Duration::from_millis(30),
            "{:?}",
            inicio.elapsed()
        );
    }
}
