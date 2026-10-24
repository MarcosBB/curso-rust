//! Exercício 3: um logger com decorador (composição + trait).

use std::fmt;

/// A ordem das variantes é a ordem de gravidade (`derive(PartialOrd)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel {
    Debug,
    Info,
    Aviso,
    Erro,
}

impl fmt::Display for Nivel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nome = match self {
            Nivel::Debug => "DEBUG",
            Nivel::Info => "INFO",
            Nivel::Aviso => "AVISO",
            Nivel::Erro => "ERRO",
        };
        f.write_str(nome)
    }
}

pub trait Logger {
    fn log(&mut self, nivel: Nivel, mensagem: &str);

    // Atalhos: métodos padrão em cima do obrigatório.
    fn info(&mut self, mensagem: &str) {
        self.log(Nivel::Info, mensagem);
    }

    fn erro(&mut self, mensagem: &str) {
        self.log(Nivel::Erro, mensagem);
    }
}

/// Escreve no stderr.
pub struct StderrLogger;

impl Logger for StderrLogger {
    fn log(&mut self, nivel: Nivel, mensagem: &str) {
        eprintln!("[{nivel}] {mensagem}");
    }
}

/// Guarda as mensagens na memória (útil em testes).
#[derive(Debug, Default)]
pub struct MemoriaLogger {
    pub mensagens: Vec<String>,
}

impl Logger for MemoriaLogger {
    fn log(&mut self, nivel: Nivel, mensagem: &str) {
        self.mensagens.push(format!("[{nivel}] {mensagem}"));
    }
}

/// Decorador: repassa para `interno` só as mensagens com nível `>= minimo`.
pub struct FiltroNivel<L: Logger> {
    pub minimo: Nivel,
    pub interno: L,
}

impl<L: Logger> Logger for FiltroNivel<L> {
    fn log(&mut self, nivel: Nivel, mensagem: &str) {
        if nivel >= self.minimo {
            self.interno.log(nivel, mensagem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memoria_formata() {
        let mut l = MemoriaLogger::default();
        l.log(Nivel::Aviso, "disco quase cheio");
        l.info("pronto");
        assert_eq!(l.mensagens, ["[AVISO] disco quase cheio", "[INFO] pronto"]);
    }

    #[test]
    fn filtro_descarta_abaixo_do_minimo() {
        let mut l = FiltroNivel {
            minimo: Nivel::Aviso,
            interno: MemoriaLogger::default(),
        };
        l.log(Nivel::Debug, "detalhe");
        l.info("normal");
        l.log(Nivel::Aviso, "cuidado");
        l.erro("falhou");
        assert_eq!(l.interno.mensagens, ["[AVISO] cuidado", "[ERRO] falhou"]);
    }

    #[test]
    fn filtros_se_empilham() {
        let mut l = FiltroNivel {
            minimo: Nivel::Info,
            interno: FiltroNivel {
                minimo: Nivel::Erro,
                interno: MemoriaLogger::default(),
            },
        };
        l.info("some no segundo filtro");
        l.erro("passa pelos dois");
        assert_eq!(l.interno.interno.mensagens, ["[ERRO] passa pelos dois"]);
    }

    #[test]
    fn funciona_como_dyn() {
        let mut loggers: Vec<Box<dyn Logger>> = vec![
            Box::new(StderrLogger),
            Box::new(FiltroNivel {
                minimo: Nivel::Erro,
                interno: StderrLogger,
            }),
        ];
        for l in &mut loggers {
            l.info("só o primeiro mostra isto");
        }
    }
}
