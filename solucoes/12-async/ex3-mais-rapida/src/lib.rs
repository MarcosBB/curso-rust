use tokio::task::JoinSet;
use tokio::time::{Duration, sleep, timeout};

#[derive(Debug, PartialEq)]
pub enum Erro {
    TodosFalharam,
    Prazo,
}

/// Um servidor de teste: responde depois de `atraso_ms`, com sucesso ou não.
#[derive(Clone)]
pub struct Servidor {
    pub nome: &'static str,
    pub atraso_ms: u64,
    pub funciona: bool,
}

pub async fn consultar(s: Servidor) -> Result<String, String> {
    sleep(Duration::from_millis(s.atraso_ms)).await;
    if s.funciona {
        Ok(format!("resposta de {}", s.nome))
    } else {
        Err(format!("{} falhou", s.nome))
    }
}

/// Consulta todos ao mesmo tempo e devolve a primeira resposta `Ok`.
/// Se todos falharem, `TodosFalharam`; se o prazo vencer antes, `Prazo`.
pub async fn mais_rapida(servidores: Vec<Servidor>, prazo: Duration) -> Result<String, Erro> {
    let mut set = JoinSet::new();
    for s in servidores {
        set.spawn(consultar(s));
    }
    let corrida = async {
        while let Some(r) = set.join_next().await {
            if let Ok(Ok(resposta)) = r {
                return Ok(resposta);
            }
        }
        Err(Erro::TodosFalharam)
    };
    // Ao sair daqui, o `set` é descartado e cancela as consultas restantes.
    timeout(prazo, corrida).await.unwrap_or(Err(Erro::Prazo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    fn s(nome: &'static str, atraso_ms: u64, funciona: bool) -> Servidor {
        Servidor {
            nome,
            atraso_ms,
            funciona,
        }
    }

    const PRAZO: Duration = Duration::from_millis(500);

    #[tokio::test(start_paused = true)]
    async fn ganha_a_mais_rapida() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 300, true), s("b", 100, true)], PRAZO).await;
        assert_eq!(r, Ok("resposta de b".into()));
        assert_eq!(inicio.elapsed(), Duration::from_millis(100));
    }

    #[tokio::test(start_paused = true)]
    async fn ignora_as_que_falham() {
        let r = mais_rapida(vec![s("a", 50, false), s("b", 200, true)], PRAZO).await;
        assert_eq!(r, Ok("resposta de b".into()));
    }

    #[tokio::test(start_paused = true)]
    async fn todas_falham() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 50, false), s("b", 80, false)], PRAZO).await;
        assert_eq!(r, Err(Erro::TodosFalharam));
        assert_eq!(inicio.elapsed(), Duration::from_millis(80));
    }

    #[tokio::test(start_paused = true)]
    async fn prazo_vence() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 900, true)], PRAZO).await;
        assert_eq!(r, Err(Erro::Prazo));
        assert_eq!(inicio.elapsed(), PRAZO);
    }

    #[tokio::test(start_paused = true)]
    async fn lista_vazia() {
        assert_eq!(mais_rapida(vec![], PRAZO).await, Err(Erro::TodosFalharam));
    }
}
