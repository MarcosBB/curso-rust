use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

static PROXIMO_ID: AtomicU64 = AtomicU64::new(1);

/// Um id novo a cada chamada, começando em 1, único entre threads.
pub fn proximo_id() -> u64 {
    PROXIMO_ID.fetch_add(1, Ordering::Relaxed)
}

/// Métricas de requisições, atualizadas por várias threads sem `Mutex`.
pub struct Metricas {
    requisicoes: AtomicU64,
    bytes: AtomicU64,
    maior: AtomicU64,
}

impl Metricas {
    pub const fn new() -> Self {
        Self {
            requisicoes: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            maior: AtomicU64::new(0),
        }
    }

    pub fn registrar(&self, bytes: u64) {
        self.requisicoes.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        self.maior.fetch_max(bytes, Ordering::Relaxed);
    }

    pub fn requisicoes(&self) -> u64 {
        self.requisicoes.load(Ordering::Relaxed)
    }

    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    pub fn maior(&self) -> u64 {
        self.maior.load(Ordering::Relaxed)
    }
}

impl Default for Metricas {
    fn default() -> Self {
        Self::new()
    }
}

/// Métricas do processo inteiro.
pub static GLOBAL: Metricas = Metricas::new();

static STATUS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (200, "OK"),
        (201, "Created"),
        (404, "Not Found"),
        (500, "Internal Server Error"),
    ])
});

/// A descrição de um código HTTP conhecido.
pub fn status_http(codigo: u16) -> Option<&'static str> {
    STATUS.get(&codigo).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn ids_unicos_entre_threads() {
        let ids: Vec<u64> = thread::scope(|s| {
            let hs: Vec<_> = (0..8)
                .map(|_| s.spawn(|| (0..1000).map(|_| proximo_id()).collect::<Vec<_>>()))
                .collect();
            hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
        });
        let unicos: HashSet<u64> = ids.iter().copied().collect();
        assert_eq!(unicos.len(), 8000);
        assert!(ids.iter().all(|&id| id >= 1));
    }

    #[test]
    fn metricas_somam_todas_as_threads() {
        let m = Metricas::new();
        thread::scope(|s| {
            for t in 0..8 {
                let m = &m;
                s.spawn(move || {
                    for i in 1..=1000 {
                        m.registrar(i + t);
                    }
                });
            }
        });
        assert_eq!(m.requisicoes(), 8000);
        // soma de (i + t) para i em 1..=1000 e t em 0..8
        assert_eq!(m.bytes(), 8 * 500_500 + 1000 * 28);
        assert_eq!(m.maior(), 1007);
    }

    #[test]
    fn global_e_status() {
        GLOBAL.registrar(10);
        assert!(GLOBAL.requisicoes() >= 1);
        assert_eq!(status_http(404), Some("Not Found"));
        assert_eq!(status_http(418), None);
    }
}
