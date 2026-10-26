use std::io;
use std::path::PathBuf;

/// Tudo o que pode dar errado.
///
/// O `Display` de cada variante diz só o contexto; a causa (o `io::Error` ou
/// o `serde_json::Error`) fica em `source()`, e o `main` imprime a cadeia
/// inteira.
#[derive(Debug, thiserror::Error)]
pub enum Erro {
    /// Argumentos errados. O `main` mostra também a linha de uso.
    #[error("{0}")]
    Uso(String),
    /// Não existe tarefa com esse id.
    #[error("não existe tarefa com id {0}")]
    NaoEncontrada(u32),
    /// Falha ao ler ou gravar o arquivo.
    #[error("não consegui acessar {}", .caminho.display())]
    Io {
        caminho: PathBuf,
        #[source]
        fonte: io::Error,
    },
    /// O arquivo existe, mas não é um JSON de tarefas válido.
    #[error("{} não é um arquivo de tarefas válido", .caminho.display())]
    Json {
        caminho: PathBuf,
        #[source]
        fonte: serde_json::Error,
    },
    /// Falha ao escrever a saída (por exemplo, um pipe fechado).
    #[error("erro ao escrever a saída")]
    Saida(#[from] io::Error),
}
