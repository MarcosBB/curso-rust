use tokio::sync::mpsc;
use tokio::time::{Duration, interval, sleep};

#[derive(Debug, Default, PartialEq)]
pub struct Relatorio {
    pub total: u64,
    pub mensagens: usize,
    pub ticks: u32,
}

/// Simula um processamento lento (uma escrita no disco, por exemplo).
async fn processar(n: u64) -> u64 {
    sleep(Duration::from_millis(5)).await;
    n * 10
}

/// Recebe e processa até o canal fechar, contando os ticks do relógio.
pub async fn coletar(mut rx: mpsc::Receiver<u64>, intervalo: Duration) -> Relatorio {
    let mut tick = interval(intervalo);
    let mut r = Relatorio::default();
    loop {
        tokio::select! {
            // Só `recv` (cancellation safe) disputa com o tick. O
            // processamento fica no corpo do braço, que nunca é cancelado.
            msg = rx.recv() => match msg {
                Some(n) => {
                    r.total += processar(n).await;
                    r.mensagens += 1;
                }
                None => return r,
            },
            _ = tick.tick() => r.ticks += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn produtor(n: u64, pausa: Duration) -> mpsc::Receiver<u64> {
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            for i in 1..=n {
                tx.send(i).await.unwrap();
                sleep(pausa).await;
            }
        });
        rx
    }

    #[tokio::test(start_paused = true)]
    async fn nenhuma_mensagem_se_perde() {
        let rx = produtor(10, Duration::from_millis(3));
        let r = coletar(rx, Duration::from_millis(2)).await;
        assert_eq!(r.mensagens, 10);
        assert_eq!(r.total, 550);
        assert!(r.ticks > 0);
    }

    #[tokio::test(start_paused = true)]
    async fn canal_vazio_termina() {
        let (tx, rx) = mpsc::channel(1);
        drop(tx);
        let r = coletar(rx, Duration::from_millis(2)).await;
        assert_eq!((r.total, r.mensagens), (0, 0));
    }

    #[tokio::test(start_paused = true)]
    async fn tick_lento() {
        let rx = produtor(5, Duration::from_millis(1));
        let r = coletar(rx, Duration::from_secs(1)).await;
        assert_eq!(r.total, 150);
        assert_eq!(r.ticks, 1); // o primeiro tick de `interval` é imediato
    }
}
