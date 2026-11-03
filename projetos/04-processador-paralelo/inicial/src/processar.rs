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
    // Etapa 3: `percorrer`, `and_then(arquivos::processar)`, e um `fold`
    // que vai registrando num `Relatorio`. No fim, `.ordenado()`.
    todo!()
}

/// Etapa 4: um pipeline de três estágios dentro de um `thread::scope`.
///
/// ```text
/// percorrer ──caminhos──▶ N workers ──resultados──▶ agregador
/// (1 thread)  sync_channel  (ler + contar)  channel    (esta thread)
/// ```
pub fn com_canais(raizes: &[PathBuf], workers: usize) -> Relatorio {
    // Etapa 4: dois canais (caminhos e resultados), o `Receiver` dos
    // caminhos dentro de um `Mutex`, e tudo dentro de `thread::scope`.
    todo!()
}

/// Etapa 5: `rayon`, com `fold` + `reduce`, num pool de `threads` threads.
pub fn com_rayon(raizes: &[PathBuf], threads: usize) -> Relatorio {
    // Etapa 5: colete os itens de `percorrer` num `Vec`, crie um pool com
    // `rayon::ThreadPoolBuilder` e, dentro de `pool.install`, use
    // `into_par_iter`, `fold` e `reduce`.
    todo!()
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
