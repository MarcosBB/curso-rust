//! Estado global seguro: atômicos, `Mutex` em `static`, `LazyLock` e
//! `OnceLock` (lição 06). Nenhum `static mut`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::thread;

/// Inicializador `const`: um atômico pode ser `static` direto.
static PROXIMO_ID: AtomicU64 = AtomicU64::new(1);

/// `Mutex::new` e `Vec::new` também são `const fn`.
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Um `HashMap` precisa de alocação: só em tempo de execução, na primeira vez.
static UFS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    println!("  (montando a tabela de UFs)");
    HashMap::from([
        ("SP", "São Paulo"),
        ("RJ", "Rio de Janeiro"),
        ("MG", "Minas Gerais"),
    ])
});

/// Configuração definida uma vez, por quem roda o programa.
#[derive(Debug)]
struct Config {
    verboso: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config { verboso: false })
}

fn novo_id() -> u64 {
    PROXIMO_ID.fetch_add(1, Ordering::Relaxed)
}

fn registrar(msg: &str) {
    if config().verboso {
        println!("log: {msg}");
    }
    LOG.lock().unwrap().push(msg.to_string());
}

fn nome_da_uf(sigla: &str) -> Option<&'static str> {
    UFS.get(sigla).copied()
}

fn main() {
    let verboso = std::env::args().any(|a| a == "-v");
    CONFIG.set(Config { verboso }).unwrap();
    println!("config: {:?}", config());
    println!(
        "segundo set: {:?}",
        CONFIG.set(Config { verboso: true }).is_err()
    );

    thread::scope(|s| {
        for t in 0..3 {
            s.spawn(move || {
                let id = novo_id();
                registrar(&format!("thread {t} pegou o id {id}"));
            });
        }
    });
    let mut log = LOG.lock().unwrap().clone();
    log.sort();
    println!("{log:#?}");

    println!("antes de usar a tabela");
    println!("SP = {:?}", nome_da_uf("SP"));
    println!("BA = {:?}", nome_da_uf("BA"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_sao_unicos_entre_threads() {
        let ids = Mutex::new(HashSet::new());
        thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..1000 {
                        assert!(ids.lock().unwrap().insert(novo_id()));
                    }
                });
            }
        });
        assert_eq!(ids.into_inner().unwrap().len(), 8000);
    }

    #[test]
    fn tabela_preguicosa() {
        assert_eq!(nome_da_uf("RJ"), Some("Rio de Janeiro"));
        assert_eq!(nome_da_uf("XX"), None);
    }

    #[test]
    fn config_padrao_se_ninguem_definiu() {
        assert!(!config().verboso);
    }
}
