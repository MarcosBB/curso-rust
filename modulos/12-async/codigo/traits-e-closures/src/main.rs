//! Lição 08: `async fn` em traits (estático, com `Send`, e com `dyn`) e
//! async closures.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tokio::time::{Duration, sleep};

// ---------- 1. async fn em trait, uso estático (genéricos) ----------

/// Uma fonte de dados. A trait é privada ao crate, então `async fn` serve.
trait Fonte {
    async fn ler(&self, chave: &str) -> Option<String>;
}

struct Memoria(HashMap<String, String>);

impl Fonte for Memoria {
    async fn ler(&self, chave: &str) -> Option<String> {
        self.0.get(chave).cloned()
    }
}

struct Lenta;

impl Fonte for Lenta {
    async fn ler(&self, chave: &str) -> Option<String> {
        sleep(Duration::from_millis(20)).await;
        Some(chave.to_uppercase())
    }
}

/// Genérico: o compilador gera uma versão por tipo, sem alocação.
async fn ler_varias<F: Fonte>(fonte: &F, chaves: &[&str]) -> Vec<Option<String>> {
    let mut saida = Vec::new();
    for c in chaves {
        saida.push(fonte.ler(c).await);
    }
    saida
}

// ---------- 2. exigindo Send: para usar com tokio::spawn ----------

/// A future de `ler` é declarada `Send`, então pode ir para outra thread.
trait FonteSend {
    fn ler(&self, chave: &str) -> impl Future<Output = Option<String>> + Send;
}

impl FonteSend for Lenta {
    // A implementação pode continuar usando `async fn`.
    async fn ler(&self, chave: &str) -> Option<String> {
        sleep(Duration::from_millis(20)).await;
        Some(chave.to_uppercase())
    }
}

async fn ler_em_task<F>(fonte: F, chave: String) -> Option<String>
where
    F: FonteSend + Send + 'static,
{
    tokio::spawn(async move { fonte.ler(&chave).await })
        .await
        .unwrap()
}

// ---------- 3. dyn: a future vira Box<dyn Future> ----------

/// O tipo de future que um método "async" dyn-compatível devolve.
type FutureBox<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait FonteDyn {
    fn ler<'a>(&'a self, chave: &'a str) -> FutureBox<'a, Option<String>>;
}

impl FonteDyn for Memoria {
    fn ler<'a>(&'a self, chave: &'a str) -> FutureBox<'a, Option<String>> {
        Box::pin(async move { self.0.get(chave).cloned() })
    }
}

impl FonteDyn for Lenta {
    fn ler<'a>(&'a self, chave: &'a str) -> FutureBox<'a, Option<String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(20)).await;
            Some(chave.to_uppercase())
        })
    }
}

/// Procura a chave em cada fonte, na ordem, e devolve o primeiro achado.
async fn primeira_que_tem(fontes: &[Box<dyn FonteDyn>], chave: &str) -> Option<String> {
    for f in fontes {
        if let Some(v) = f.ler(chave).await {
            return Some(v);
        }
    }
    None
}

// ---------- 4. async closures ----------

/// Chama `f` para cada nome. `AsyncFn` aceita closures que emprestam o argumento.
async fn para_cada<F>(nomes: &[String], f: F)
where
    F: AsyncFn(&String),
{
    for n in nomes {
        f(n).await;
    }
}

/// Tenta `op` até `tentativas` vezes, devolvendo o primeiro `Ok`.
async fn com_retentativa<T, E, F>(tentativas: u32, mut op: F) -> Result<T, E>
where
    F: AsyncFnMut(u32) -> Result<T, E>,
{
    let mut tentativa = 1;
    loop {
        match op(tentativa).await {
            Ok(v) => return Ok(v),
            Err(e) if tentativa >= tentativas => return Err(e),
            Err(_) => tentativa += 1,
        }
    }
}

fn memoria() -> Memoria {
    Memoria(HashMap::from([("nome".to_string(), "Ana".to_string())]))
}

#[tokio::main]
async fn main() {
    println!("== genérico");
    println!("{:?}", ler_varias(&memoria(), &["nome", "idade"]).await);
    println!("{:?}", ler_varias(&Lenta, &["nome"]).await);

    println!("\n== com Send, dentro de tokio::spawn");
    println!("{:?}", ler_em_task(Lenta, "oi".into()).await);

    println!("\n== dyn");
    let fontes: Vec<Box<dyn FonteDyn>> = vec![Box::new(memoria()), Box::new(Lenta)];
    println!("{:?}", primeira_que_tem(&fontes, "nome").await);
    println!("{:?}", primeira_que_tem(&fontes, "cidade").await);

    println!("\n== async closures");
    let nomes = vec![String::from("ana"), String::from("bia")];
    let saudacao = String::from("olá");
    para_cada(&nomes, async |n| println!("{saudacao}, {n}")).await;

    let mut chamadas = Vec::new();
    let r: Result<&str, String> = com_retentativa(5, async |t| {
        chamadas.push(t);
        sleep(Duration::from_millis(5)).await;
        if t < 3 {
            Err(format!("falhou na {t}"))
        } else {
            Ok("conectou")
        }
    })
    .await;
    println!("{r:?} depois das tentativas {chamadas:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generico_com_duas_fontes() {
        assert_eq!(
            ler_varias(&memoria(), &["nome", "x"]).await,
            [Some("Ana".to_string()), None]
        );
        assert_eq!(ler_varias(&Lenta, &["a"]).await, [Some("A".to_string())]);
    }

    #[tokio::test]
    async fn dyn_procura_em_ordem() {
        let fontes: Vec<Box<dyn FonteDyn>> = vec![Box::new(memoria()), Box::new(Lenta)];
        assert_eq!(primeira_que_tem(&fontes, "nome").await, Some("Ana".into()));
        assert_eq!(primeira_que_tem(&fontes, "x").await, Some("X".into()));
    }

    #[tokio::test]
    async fn retentativa_desiste() {
        let r: Result<(), u32> = com_retentativa(3, async |t| Err(t)).await;
        assert_eq!(r, Err(3));
    }
}
