//! Os três modos de processar: sequencial, threads com canais, `rayon`.
//!
//! Os três recebem as mesmas raízes e devolvem o mesmo [`Relatorio`]
//! (depois de [`Relatorio::ordenado`]).

use std::path::PathBuf;
use std::sync::{Mutex, mpsc};
use std::thread;

use rayon::prelude::*;

use crate::arquivos::{self, ErroArquivo};
use crate::contagem::Contagem;
use crate::relatorio::Relatorio;

/// O que cada worker manda para o agregador.
type Resultado = Result<(PathBuf, Contagem), ErroArquivo>;

/// Quantas threads usar quando o usuário pede `0` ("automático").
pub fn threads_automatico(pedidas: usize) -> usize {
    if pedidas > 0 {
        return pedidas;
    }
    thread::available_parallelism().map_or(1, |n| n.get())
}

/// Etapa 3: um arquivo depois do outro, numa thread só.
pub fn sequencial(raizes: &[PathBuf]) -> Relatorio {
    arquivos::percorrer(raizes)
        .map(|item| item.and_then(arquivos::processar))
        .fold(Relatorio::default(), |mut r, resultado| {
            r.registrar(resultado);
            r
        })
        .ordenado()
}

/// Etapa 4: um pipeline de três estágios dentro de um `thread::scope`.
///
/// ```text
/// percorrer ──caminhos──▶ N workers ──resultados──▶ agregador
/// (1 thread)  sync_channel  (ler + contar)  channel    (esta thread)
/// ```
pub fn com_canais(raizes: &[PathBuf], workers: usize) -> Relatorio {
    let workers = threads_automatico(workers);
    // Fila limitada: se o disco listar mais rápido do que os workers leem,
    // o `send` do primeiro estágio espera, e a memória não cresce à toa.
    let (tx_caminhos, rx_caminhos) = mpsc::sync_channel::<PathBuf>(64);
    let (tx_resultados, rx_resultados) = mpsc::channel::<Resultado>();
    // O `Receiver` não é `Clone`: os workers dividem um só, cada um travando
    // o `Mutex` só pelo tempo de tirar um caminho da fila.
    let rx_caminhos = Mutex::new(rx_caminhos);

    thread::scope(|s| {
        // Estágio 1: percorre os diretórios. Erros de listagem vão direto
        // para o agregador; caminhos vão para os workers.
        let tx_erros = tx_resultados.clone();
        s.spawn(move || {
            for item in arquivos::percorrer(raizes) {
                let enviado = match item {
                    Ok(caminho) => tx_caminhos.send(caminho).is_ok(),
                    Err(e) => tx_erros.send(Err(e)).is_ok(),
                };
                if !enviado {
                    break; // ninguém mais escuta: não adianta continuar
                }
            }
            // Aqui `tx_caminhos` é descartado, e os workers saem do laço.
        });

        // Estágio 2: os workers. Dentro do `scope`, basta emprestar o
        // `Mutex` (`&rx_caminhos`); não precisa de `Arc`.
        let rx_caminhos = &rx_caminhos;
        for _ in 0..workers {
            let tx = tx_resultados.clone();
            s.spawn(move || {
                loop {
                    // O guarda do `lock()` é um temporário deste `let`:
                    // destrava aqui, antes de ler o arquivo. Com
                    // `while let Ok(c) = rx.lock().unwrap().recv()`, ele
                    // ficaria travado o corpo inteiro, e um worker por vez
                    // trabalharia.
                    let Ok(caminho) = rx_caminhos.lock().unwrap().recv() else {
                        break;
                    };
                    if tx.send(arquivos::processar(caminho)).is_err() {
                        break;
                    }
                }
            });
        }

        // Sem este `drop`, o `for` abaixo nunca termina: o `tx_resultados`
        // original ainda existiria, e o canal nunca "fecharia".
        drop(tx_resultados);

        // Estágio 3: o agregador, nesta thread.
        let mut relatorio = Relatorio::default();
        for resultado in rx_resultados {
            relatorio.registrar(resultado);
        }
        relatorio
    })
    .ordenado()
}

/// Etapa 5: `rayon`, com `fold` + `reduce`, num pool de `threads` threads.
pub fn com_rayon(raizes: &[PathBuf], threads: usize) -> Relatorio {
    // Listar primeiro: um `Vec` se divide bem entre as threads.
    let itens: Vec<Result<PathBuf, ErroArquivo>> = arquivos::percorrer(raizes).collect();

    // `num_threads(0)` deixa o rayon escolher (uma por núcleo).
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("criar o pool só falha se o sistema não deixar criar threads");

    pool.install(|| {
        itens
            .into_par_iter()
            .map(|item| item.and_then(arquivos::processar))
            .fold(Relatorio::default, |mut r, resultado| {
                r.registrar(resultado);
                r
            })
            .reduce(Relatorio::default, Relatorio::juntar)
    })
    .ordenado()
}

#[cfg(test)]
mod testes_comuns {
    use super::*;
    use crate::relatorio::Arquivo;
    use std::fs;

    /// 60 arquivos em 3 níveis, um binário e uma raiz que não existe.
    pub fn corpus() -> (tempfile::TempDir, Vec<PathBuf>) {
        let dir = tempfile::tempdir().unwrap();
        let palavras = ["rust", "thread", "canal", "rayon", "mutex", "arc", "send"];
        for i in 0..60 {
            let sub = dir.path().join(format!("d{}/e{}", i % 3, i % 5));
            fs::create_dir_all(&sub).unwrap();
            let texto: String = (0..i * 7)
                .map(|j| palavras[(i + j * j) % palavras.len()])
                .collect::<Vec<_>>()
                .chunks(5)
                .map(|linha| linha.join(" ") + "\n")
                .collect();
            fs::write(sub.join(format!("f{i:02}.txt")), texto).unwrap();
        }
        fs::write(dir.path().join("d1/imagem.bin"), [0xff, 0xd8, 0xff]).unwrap();
        let raizes = vec![dir.path().to_path_buf(), dir.path().join("sumiu")];
        (dir, raizes)
    }

    /// O que dá para comparar com `==` (o `io::Error` não é `PartialEq`).
    pub fn resumo(r: &Relatorio) -> (&[Arquivo], &Contagem, Vec<String>) {
        let erros = r.erros.iter().map(ToString::to_string).collect();
        (&r.arquivos, &r.total, erros)
    }
}

#[cfg(test)]
mod step3_tests {
    use super::testes_comuns::corpus;
    use super::*;
    use std::path::Path;

    #[test]
    fn sequencial_no_corpus() {
        let (dir, raizes) = corpus();
        let r = sequencial(&raizes);
        assert_eq!(r.arquivos.len(), 60);
        assert_eq!(r.total.palavras, (0..60).map(|i| i * 7).sum::<usize>());
        let erros: Vec<&Path> = r.erros.iter().map(ErroArquivo::caminho).collect();
        assert_eq!(
            erros,
            [dir.path().join("d1/imagem.bin"), dir.path().join("sumiu")]
        );
        assert!(r.arquivos.is_sorted_by(|a, b| a.caminho <= b.caminho));
    }
}

#[cfg(test)]
mod step4_tests {
    use super::testes_comuns::{corpus, resumo};
    use super::*;

    #[test]
    fn canais_igual_ao_sequencial() {
        let (_dir, raizes) = corpus();
        let esperado = sequencial(&raizes);
        for workers in [1, 2, 8] {
            let r = com_canais(&raizes, workers);
            assert_eq!(resumo(&r), resumo(&esperado), "com {workers} workers");
        }
    }

    #[test]
    fn canais_com_workers_automatico() {
        let (_dir, raizes) = corpus();
        assert_eq!(
            resumo(&com_canais(&raizes, 0)),
            resumo(&sequencial(&raizes))
        );
    }

    #[test]
    fn canais_sem_nada_para_ler() {
        let dir = tempfile::tempdir().unwrap();
        let r = com_canais(&[dir.path().to_path_buf()], 4);
        assert!(r.arquivos.is_empty() && r.erros.is_empty());
    }
}

#[cfg(test)]
mod step5_tests {
    use super::testes_comuns::{corpus, resumo};
    use super::*;

    #[test]
    fn rayon_igual_ao_sequencial() {
        let (_dir, raizes) = corpus();
        let esperado = sequencial(&raizes);
        for threads in [0, 1, 4] {
            let r = com_rayon(&raizes, threads);
            assert_eq!(resumo(&r), resumo(&esperado), "com {threads} threads");
        }
    }

    #[test]
    fn os_tres_modos_concordam() {
        let (_dir, raizes) = corpus();
        let seq = sequencial(&raizes);
        let canais = com_canais(&raizes, 3);
        let rayon = com_rayon(&raizes, 3);
        assert_eq!(resumo(&seq), resumo(&canais));
        assert_eq!(resumo(&seq), resumo(&rayon));
        assert_eq!(seq.total.mais_frequentes(3), rayon.total.mais_frequentes(3));
    }
}
