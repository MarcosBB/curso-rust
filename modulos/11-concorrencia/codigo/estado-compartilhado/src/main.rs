//! Estado compartilhado: `Arc<Mutex<T>>`, `RwLock`, *poisoning* (lição 04)
//! e atômicos (lição 05).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

/// Conta palavras de vários textos, uma thread por texto, num mapa protegido
/// por um `Mutex`. Cada thread conta localmente e trava o mapa uma vez só.
fn contar_palavras(textos: &[&str]) -> HashMap<String, usize> {
    let total = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();
    for texto in textos {
        let total = Arc::clone(&total);
        let texto = texto.to_string();
        handles.push(thread::spawn(move || {
            let mut local: HashMap<String, usize> = HashMap::new();
            for p in texto.split_whitespace() {
                *local.entry(p.to_lowercase()).or_default() += 1;
            }
            let mut mapa = total.lock().unwrap();
            for (p, n) in local {
                *mapa.entry(p).or_default() += n;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    // Todas as threads terminaram: este é o último Arc, e dá para tirar o valor.
    Arc::try_unwrap(total).unwrap().into_inner().unwrap()
}

/// Uma configuração lida por muitos e trocada raramente: `RwLock`.
struct Config {
    dados: RwLock<HashMap<String, String>>,
}

impl Config {
    fn ler(&self, chave: &str) -> Option<String> {
        self.dados.read().unwrap().get(chave).cloned()
    }

    fn gravar(&self, chave: &str, valor: &str) {
        self.dados
            .write()
            .unwrap()
            .insert(chave.to_string(), valor.to_string());
    }
}

/// Um contador atômico: sem lock, sem `&mut`.
fn contar_com_atomico(threads: usize, por_thread: u64) -> u64 {
    let contador = AtomicU64::new(0);
    thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                for _ in 0..por_thread {
                    contador.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    contador.into_inner()
}

/// Uma thread trabalha até alguém pedir para parar.
fn trabalhar_ate_parar() -> u64 {
    let parar = AtomicBool::new(false);
    let voltas = AtomicU64::new(0);
    thread::scope(|s| {
        s.spawn(|| {
            while !parar.load(Ordering::Relaxed) {
                voltas.fetch_add(1, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(1));
            }
        });
        thread::sleep(Duration::from_millis(20));
        parar.store(true, Ordering::Relaxed);
    });
    voltas.into_inner()
}

/// Publicar um dado com Release/Acquire: quem vê `pronto == true` vê o dado.
fn publicar() -> u64 {
    let dado = AtomicU64::new(0);
    let pronto = AtomicBool::new(false);
    thread::scope(|s| {
        s.spawn(|| {
            dado.store(42, Ordering::Relaxed);
            pronto.store(true, Ordering::Release);
        });
        while !pronto.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
        dado.load(Ordering::Relaxed)
    })
}

fn main() {
    let contagem = contar_palavras(&["o gato e o rato", "O rato", "gato gato"]);
    let mut pares: Vec<_> = contagem.into_iter().collect();
    pares.sort();
    println!("{pares:?}");

    let config = Config {
        dados: RwLock::new(HashMap::new()),
    };
    config.gravar("idioma", "pt-BR");
    thread::scope(|s| {
        for i in 0..3 {
            let config = &config;
            s.spawn(move || println!("leitor {i}: {:?}", config.ler("idioma")));
        }
    });

    // Poisoning: uma thread entra em pânico segurando o lock.
    let saldo = Arc::new(Mutex::new(100));
    let s = Arc::clone(&saldo);
    let _ = thread::spawn(move || {
        let mut g = s.lock().unwrap();
        *g -= 30;
        panic!("falhou no meio da transferência");
    })
    .join();
    match saldo.lock() {
        Ok(g) => println!("saldo: {g}"),
        Err(envenenado) => {
            println!("mutex envenenado: {envenenado}");
            println!("valor lá dentro: {}", envenenado.into_inner());
        }
    }

    println!("atômico: {}", contar_com_atomico(8, 100_000));
    println!("voltas até parar: algumas ({})", trabalhar_ate_parar() > 0);
    println!("publicado: {}", publicar());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contagem_soma_todos_os_textos() {
        let c = contar_palavras(&["a b a", "B c", ""]);
        assert_eq!(c["a"], 2);
        assert_eq!(c["b"], 2);
        assert_eq!(c["c"], 1);
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn config_le_o_que_gravou() {
        let c = Config {
            dados: RwLock::new(HashMap::new()),
        };
        assert_eq!(c.ler("x"), None);
        c.gravar("x", "1");
        assert_eq!(c.ler("x").as_deref(), Some("1"));
    }

    #[test]
    fn atomico_nao_perde_incrementos() {
        assert_eq!(contar_com_atomico(8, 10_000), 80_000);
    }

    #[test]
    fn parar_e_publicar() {
        assert!(trabalhar_ate_parar() > 0);
        assert_eq!(publicar(), 42);
    }
}
