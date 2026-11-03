//! O resultado de processar muitos arquivos, e como ele é impresso.

use std::io::{self, Write};
use std::path::PathBuf;

use crate::arquivos::ErroArquivo;
use crate::contagem::Contagem;

/// Linhas e palavras de um arquivo (as frequências vão só para o total).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arquivo {
    pub caminho: PathBuf,
    pub linhas: usize,
    pub palavras: usize,
}

/// Tudo o que um modo de processamento devolve.
#[derive(Debug, Default)]
pub struct Relatorio {
    pub arquivos: Vec<Arquivo>,
    pub total: Contagem,
    pub erros: Vec<ErroArquivo>,
}

impl Relatorio {
    /// Acrescenta o resultado de um arquivo: a contagem ou o erro.
    pub fn registrar(&mut self, resultado: Result<(PathBuf, Contagem), ErroArquivo>) {
        // Etapa 3.
        todo!()
    }

    /// Junta dois relatórios parciais num só. Recebe e devolve por valor,
    /// no formato que o `reduce` do rayon pede.
    pub fn juntar(mut self, outro: Relatorio) -> Relatorio {
        // Etapa 3.
        todo!()
    }

    /// Ordena arquivos e erros pelo caminho. Threads terminam em qualquer
    /// ordem; depois disto, os três modos dão relatórios iguais.
    pub fn ordenado(mut self) -> Relatorio {
        // Etapa 3: `sort_by` comparando os caminhos.
        todo!()
    }

    /// Escreve o relatório. Com `por_arquivo`, uma linha por arquivo antes
    /// do resumo. Os erros não entram aqui: o `main` os manda para o stderr.
    ///
    /// # Errors
    ///
    /// Só os de escrita em `saida`.
    pub fn escrever(
        &self,
        saida: &mut impl Write,
        top: usize,
        por_arquivo: bool,
    ) -> io::Result<()> {
        if por_arquivo {
            writeln!(saida, "{:>8} {:>9}  arquivo", "linhas", "palavras")?;
            for a in &self.arquivos {
                writeln!(
                    saida,
                    "{:>8} {:>9}  {}",
                    a.linhas,
                    a.palavras,
                    a.caminho.display()
                )?;
            }
            writeln!(saida)?;
        }
        writeln!(
            saida,
            "arquivos: {} lidos, {} com erro",
            self.arquivos.len(),
            self.erros.len()
        )?;
        writeln!(saida, "linhas:   {}", self.total.linhas)?;
        writeln!(saida, "palavras: {}", self.total.palavras)?;
        let mais = self.total.mais_frequentes(top);
        if !mais.is_empty() {
            writeln!(saida, "mais frequentes:")?;
            for (p, n) in mais {
                writeln!(saida, "{n:>8}  {p}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod step3_tests {
    use super::*;
    use crate::contagem::contar;
    use std::path::Path;

    fn ok(caminho: &str, texto: &str) -> Result<(PathBuf, Contagem), ErroArquivo> {
        Ok((PathBuf::from(caminho), contar(texto)))
    }

    fn nao_utf8(caminho: &str) -> Result<(PathBuf, Contagem), ErroArquivo> {
        Err(ErroArquivo::NaoUtf8 {
            caminho: PathBuf::from(caminho),
        })
    }

    #[test]
    fn registrar_separa_contagens_e_erros() {
        let mut r = Relatorio::default();
        r.registrar(ok("a.txt", "um dois\ntrês"));
        r.registrar(nao_utf8("x.bin"));
        assert_eq!(
            r.arquivos,
            [Arquivo {
                caminho: "a.txt".into(),
                linhas: 2,
                palavras: 3
            }]
        );
        assert_eq!(r.total.palavras, 3);
        assert_eq!(r.erros.len(), 1);
    }

    #[test]
    fn juntar_e_ordenar() {
        let mut a = Relatorio::default();
        a.registrar(ok("c.txt", "x"));
        a.registrar(nao_utf8("z.bin"));
        let mut b = Relatorio::default();
        b.registrar(ok("a.txt", "x y"));
        b.registrar(nao_utf8("m.bin"));
        let r = a.juntar(b).ordenado();
        let nomes: Vec<&Path> = r.arquivos.iter().map(|a| a.caminho.as_path()).collect();
        assert_eq!(nomes, [Path::new("a.txt"), Path::new("c.txt")]);
        let erros: Vec<&Path> = r.erros.iter().map(ErroArquivo::caminho).collect();
        assert_eq!(erros, [Path::new("m.bin"), Path::new("z.bin")]);
        assert_eq!(r.total.frequencias["x"], 2);
    }

    #[test]
    fn formato_da_saida() {
        let mut r = Relatorio::default();
        r.registrar(ok("a.txt", "o rato roeu\na roupa do rei"));
        r.registrar(ok("b.txt", "o rei"));
        r.registrar(nao_utf8("c.bin"));
        let mut saida = Vec::new();
        r.escrever(&mut saida, 2, true).unwrap();
        let esperado = [
            "  linhas  palavras  arquivo",
            "       2         7  a.txt",
            "       1         2  b.txt",
            "",
            "arquivos: 2 lidos, 1 com erro",
            "linhas:   3",
            "palavras: 9",
            "mais frequentes:",
            "       2  o",
            "       2  rei",
        ];
        assert_eq!(
            String::from_utf8(saida).unwrap(),
            esperado.join("\n") + "\n"
        );
    }
}
