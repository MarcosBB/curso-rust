//! `contapal`: conta linhas, palavras e as palavras mais frequentes de todos
//! os arquivos de um ou mais diretórios, de três jeitos: sequencial, com
//! threads e canais, e com `rayon`.
//!
//! A biblioteca não imprime nada e não encerra o programa. Os três modos
//! devolvem o mesmo [`Relatorio`], e é isso que os testes comparam.

pub mod arquivos;
pub mod config;
pub mod contagem;
pub mod processar;
pub mod relatorio;

pub use arquivos::ErroArquivo;
pub use contagem::Contagem;
pub use relatorio::Relatorio;
