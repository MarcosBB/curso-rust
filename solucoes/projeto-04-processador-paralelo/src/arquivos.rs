//! Encontrar os arquivos e ler cada um, com um erro por arquivo.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use walkdir::WalkDir;

use crate::contagem::{Contagem, contar};

/// Um problema com **um** arquivo (ou diretório). Não é fatal: o relatório
/// guarda o erro e os outros arquivos continuam sendo processados.
#[derive(Debug, Error)]
pub enum ErroArquivo {
    /// Não deu para listar o diretório ou ler o arquivo.
    #[error("{}: {fonte}", caminho.display())]
    Io {
        caminho: PathBuf,
        #[source]
        fonte: io::Error,
    },
    /// O arquivo existe, mas não é texto UTF-8.
    #[error("{}: o arquivo não é UTF-8 válido", caminho.display())]
    NaoUtf8 { caminho: PathBuf },
}

impl ErroArquivo {
    /// O caminho do arquivo com problema (usado para ordenar os erros).
    pub fn caminho(&self) -> &Path {
        match self {
            ErroArquivo::Io { caminho, .. } | ErroArquivo::NaoUtf8 { caminho } => caminho,
        }
    }
}

/// Todos os arquivos comuns dentro de `raizes`, recursivamente.
///
/// Diretórios são percorridos em ordem alfabética. Links simbólicos não são
/// seguidos (o padrão do `walkdir`), então não há risco de laço infinito.
/// Um diretório que não abre vira um `Err` no meio da sequência, e o resto
/// continua.
pub fn percorrer(raizes: &[PathBuf]) -> impl Iterator<Item = Result<PathBuf, ErroArquivo>> {
    raizes.iter().flat_map(|raiz| {
        WalkDir::new(raiz)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|item| match item {
                Ok(entrada) if entrada.file_type().is_file() => Some(Ok(entrada.into_path())),
                Ok(_) => None, // diretório ou link simbólico
                Err(e) => {
                    let caminho = e.path().unwrap_or(raiz).to_path_buf();
                    // O `io::Error` de dentro, sem o texto do walkdir, que
                    // repetiria o caminho. Só não há um quando o walkdir acha
                    // um laço de links, o que não acontece sem `follow_links`.
                    let fonte = e
                        .into_io_error()
                        .unwrap_or_else(|| io::Error::other("laço de links simbólicos"));
                    Some(Err(ErroArquivo::Io { caminho, fonte }))
                }
            })
    })
}

/// Lê um arquivo inteiro como texto.
pub fn ler(caminho: &Path) -> Result<String, ErroArquivo> {
    fs::read_to_string(caminho).map_err(|fonte| {
        // `read_to_string` avisa "não é UTF-8" com `InvalidData`.
        if fonte.kind() == io::ErrorKind::InvalidData {
            ErroArquivo::NaoUtf8 {
                caminho: caminho.to_path_buf(),
            }
        } else {
            ErroArquivo::Io {
                caminho: caminho.to_path_buf(),
                fonte,
            }
        }
    })
}

/// Lê e conta um arquivo. É o trabalho que os três modos repartem.
pub fn processar(caminho: PathBuf) -> Result<(PathBuf, Contagem), ErroArquivo> {
    let texto = ler(&caminho)?;
    Ok((caminho, contar(&texto)))
}

#[cfg(test)]
mod step2_tests {
    use super::*;

    fn arvore() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        fs::create_dir_all(d.join("b/sub")).unwrap();
        fs::write(d.join("z.txt"), "z").unwrap();
        fs::write(d.join("a.txt"), "a").unwrap();
        fs::write(d.join("b/sub/fundo.txt"), "fundo").unwrap();
        fs::write(d.join("b/meio.txt"), "meio").unwrap();
        dir
    }

    #[test]
    fn percorre_recursivamente_em_ordem() {
        let dir = arvore();
        let achados: Vec<PathBuf> = percorrer(&[dir.path().to_path_buf()])
            .map(|r| r.unwrap().strip_prefix(dir.path()).unwrap().to_path_buf())
            .collect();
        let esperado: Vec<PathBuf> = ["a.txt", "b/meio.txt", "b/sub/fundo.txt", "z.txt"]
            .iter()
            .map(PathBuf::from)
            .collect();
        assert_eq!(achados, esperado);
    }

    #[test]
    fn raiz_que_e_um_arquivo() {
        let dir = arvore();
        let raiz = dir.path().join("a.txt");
        let achados: Vec<_> = percorrer(std::slice::from_ref(&raiz)).collect();
        assert_eq!(achados.len(), 1);
        assert_eq!(achados[0].as_ref().unwrap(), &raiz);
    }

    #[test]
    fn raiz_inexistente_vira_erro_e_nao_para_as_outras() {
        let dir = arvore();
        let raizes = [dir.path().join("nao-existe"), dir.path().join("b")];
        let itens: Vec<_> = percorrer(&raizes).collect();
        assert_eq!(itens.len(), 3);
        let erro = itens[0].as_ref().unwrap_err();
        assert_eq!(erro.caminho(), dir.path().join("nao-existe"));
        assert!(
            matches!(erro, ErroArquivo::Io { fonte, .. } if fonte.kind() == io::ErrorKind::NotFound)
        );
        assert!(itens[1..].iter().all(Result::is_ok));
    }

    #[test]
    fn arquivo_que_nao_e_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let caminho = dir.path().join("imagem.bin");
        fs::write(&caminho, [0xff, 0xfe, 0x00, 0x41]).unwrap();
        let erro = ler(&caminho).unwrap_err();
        assert!(matches!(erro, ErroArquivo::NaoUtf8 { .. }));
        assert_eq!(
            erro.to_string(),
            format!("{}: o arquivo não é UTF-8 válido", caminho.display())
        );
    }

    #[test]
    fn processar_le_e_conta() {
        let dir = arvore();
        let (caminho, c) = processar(dir.path().join("b/meio.txt")).unwrap();
        assert!(caminho.ends_with("meio.txt"));
        assert_eq!((c.linhas, c.palavras), (1, 1));
    }
}
