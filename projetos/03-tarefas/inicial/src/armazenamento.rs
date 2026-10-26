use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::Erro;
use crate::lista::ListaDeTarefas;

/// Onde a lista fica guardada.
///
/// O programa usa [`ArquivoJson`]; os testes usam [`Memoria`]. Quem recebe
/// um `&mut impl Armazenamento` não sabe (nem precisa saber) qual é.
pub trait Armazenamento {
    /// Lê a lista guardada (uma lista vazia, se ainda não há nada).
    ///
    /// # Errors
    ///
    /// Quando não dá para ler o que está guardado.
    fn carregar(&self) -> Result<ListaDeTarefas, Erro>;

    /// Guarda a lista, substituindo a anterior.
    ///
    /// # Errors
    ///
    /// Quando não dá para gravar.
    fn salvar(&mut self, lista: &ListaDeTarefas) -> Result<(), Erro>;
}

/// A lista num arquivo JSON.
#[derive(Debug, Clone)]
pub struct ArquivoJson {
    caminho: PathBuf,
}

impl ArquivoJson {
    /// Usa o arquivo em `caminho`. Nada é lido nem criado aqui.
    pub fn new(caminho: impl Into<PathBuf>) -> Self {
        ArquivoJson {
            caminho: caminho.into(),
        }
    }

    /// O caminho do arquivo.
    pub fn caminho(&self) -> &Path {
        &self.caminho
    }

    fn io(&self, fonte: io::Error) -> Erro {
        Erro::Io {
            caminho: self.caminho.clone(),
            fonte,
        }
    }
}

/// `tarefas.json` -> `tarefas.json.tmp`, na mesma pasta.
fn temporario(caminho: &Path) -> PathBuf {
    // Etapa 4.
    todo!()
}

impl Armazenamento for ArquivoJson {
    fn carregar(&self) -> Result<ListaDeTarefas, Erro> {
        // Etapa 4: arquivo inexistente (`io::ErrorKind::NotFound`) = lista vazia.
        todo!()
    }

    /// Grava num arquivo temporário e renomeia por cima do original. Se o
    /// programa cair no meio da gravação, o arquivo antigo continua inteiro.
    fn salvar(&mut self, lista: &ListaDeTarefas) -> Result<(), Erro> {
        // Etapa 4: grave em `temporario(..)` e renomeie com `fs::rename`.
        todo!()
    }
}

/// Armazenamento em memória, para testes. Conta as gravações.
#[derive(Debug, Clone, Default)]
pub struct Memoria {
    /// O que está "guardado".
    pub lista: ListaDeTarefas,
    /// Quantas vezes `salvar` foi chamado.
    pub gravacoes: usize,
}

impl Armazenamento for Memoria {
    fn carregar(&self) -> Result<ListaDeTarefas, Erro> {
        // Etapa 4.
        todo!()
    }

    fn salvar(&mut self, lista: &ListaDeTarefas) -> Result<(), Erro> {
        // Etapa 4: guarde uma cópia e conte a gravação.
        todo!()
    }
}

#[cfg(test)]
mod step4_tests {
    use super::*;
    use crate::tarefa::Prioridade;
    use std::assert_matches;

    #[test]
    fn arquivo_inexistente_e_lista_vazia() {
        let dir = tempfile::tempdir().unwrap();
        let arquivo = ArquivoJson::new(dir.path().join("nao-existe.json"));
        assert_eq!(arquivo.carregar().unwrap(), ListaDeTarefas::default());
    }

    #[test]
    fn json_quebrado_vira_erro_com_posicao() {
        let dir = tempfile::tempdir().unwrap();
        let caminho = dir.path().join("t.json");
        fs::write(&caminho, "{\n  \"versao\": ,\n}").unwrap();
        let erro = ArquivoJson::new(&caminho).carregar().unwrap_err();
        assert_matches!(&erro, Erro::Json { .. });
        let causa = std::error::Error::source(&erro).unwrap();
        assert_eq!(causa.to_string(), "expected value at line 2 column 13");
    }

    #[test]
    fn ida_e_volta_no_disco() {
        let dir = tempfile::tempdir().unwrap();
        let caminho = dir.path().join("t.json");
        let mut arquivo = ArquivoJson::new(&caminho);

        let mut lista = arquivo.carregar().unwrap();
        lista.adicionar("comprar pão".into(), Prioridade::Baixa, 10);
        arquivo.salvar(&lista).unwrap();

        assert_eq!(arquivo.carregar().unwrap(), lista);
        assert!(!dir.path().join("t.json.tmp").exists(), "o .tmp sobrou");
    }

    #[test]
    fn memoria_conta_gravacoes() {
        let mut memoria = Memoria::default();
        let mut lista = memoria.carregar().unwrap();
        lista.adicionar("a".into(), Prioridade::Media, 0);
        memoria.salvar(&lista).unwrap();
        assert_eq!(memoria.gravacoes, 1);
        assert_eq!(memoria.carregar().unwrap(), lista);
    }
}
