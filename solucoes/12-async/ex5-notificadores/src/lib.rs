use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use tokio::time::{Duration, sleep};

/// A future que um método "async" dyn-compatível devolve: alocada no heap,
/// `Send` (pode ir para outra thread) e presa ao empréstimo `'a`.
pub type FutureBox<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// `Send + Sync` como supertraits: `dyn Notificador` também é `Send + Sync`.
pub trait Notificador: Send + Sync {
    fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>>;
}

/// Guarda as mensagens "enviadas" (num servidor de verdade, seria SMTP).
#[derive(Default)]
pub struct Email {
    pub enviados: Mutex<Vec<String>>,
}

impl Notificador for Email {
    fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(20)).await;
            self.enviados.lock().unwrap().push(msg.to_string());
            Ok(())
        })
    }
}

/// Recusa mensagens maiores que `limite` caracteres.
pub struct Sms {
    pub limite: usize,
}

impl Notificador for Sms {
    fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(10)).await;
            if msg.chars().count() > self.limite {
                Err(format!(
                    "SMS: mensagem com mais de {} caracteres",
                    self.limite
                ))
            } else {
                Ok(())
            }
        })
    }
}

/// Envia `msg` por todos os canais, um depois do outro, e devolve os resultados.
pub async fn notificar_todos(
    canais: &[Box<dyn Notificador>],
    msg: &str,
) -> Vec<Result<(), String>> {
    let mut resultados = Vec::with_capacity(canais.len());
    for c in canais {
        resultados.push(c.enviar(msg).await);
    }
    resultados
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Um notificador de teste que compartilha o registro com o teste.
    struct Espiao(Arc<Mutex<Vec<String>>>);

    impl Notificador for Espiao {
        fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>> {
            Box::pin(async move {
                self.0.lock().unwrap().push(msg.to_string());
                Ok(())
            })
        }
    }

    #[tokio::test(start_paused = true)]
    async fn email_e_sms() {
        let canais: Vec<Box<dyn Notificador>> =
            vec![Box::new(Email::default()), Box::new(Sms { limite: 5 })];
        let r = notificar_todos(&canais, "oi").await;
        assert_eq!(r, [Ok(()), Ok(())]);
        let r = notificar_todos(&canais, "mensagem longa").await;
        assert_eq!(r[0], Ok(()));
        assert_eq!(r[1], Err("SMS: mensagem com mais de 5 caracteres".into()));
    }

    #[tokio::test(start_paused = true)]
    async fn roda_dentro_de_spawn() {
        let registro = Arc::new(Mutex::new(Vec::new()));
        let canais: Vec<Box<dyn Notificador>> = vec![Box::new(Espiao(registro.clone()))];
        let r = tokio::spawn(async move { notificar_todos(&canais, "olá").await })
            .await
            .unwrap();
        assert_eq!(r, [Ok(())]);
        assert_eq!(*registro.lock().unwrap(), ["olá"]);
    }
}
