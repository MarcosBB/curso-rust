//! Os argumentos da linha de comando. Este arquivo vem pronto: o parser
//! segue o do Projeto 1, e o assunto deste projeto é outro.

use std::path::PathBuf;
use std::str::FromStr;

pub const USO: &str =
    "uso: contapal [-m seq|canais|rayon] [-j THREADS] [-n TOP] [--arquivos] [--tempo] CAMINHO...";

/// Qual das três implementações usar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    Sequencial,
    Canais,
    #[default]
    Rayon,
}

impl FromStr for Modo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seq" => Ok(Modo::Sequencial),
            "canais" => Ok(Modo::Canais),
            "rayon" => Ok(Modo::Rayon),
            _ => Err(format!("modo inválido: {s} (use seq, canais ou rayon)")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub caminhos: Vec<PathBuf>,
    pub modo: Modo,
    /// Workers (modo `canais`) ou threads do pool (modo `rayon`).
    /// `0` quer dizer "uma por núcleo".
    pub threads: usize,
    /// Quantas palavras mais frequentes mostrar.
    pub top: usize,
    /// Mostrar uma linha por arquivo antes do resumo.
    pub por_arquivo: bool,
    /// Mostrar no stderr quanto tempo o processamento levou.
    pub tempo: bool,
}

impl Config {
    /// Interpreta os argumentos (sem o nome do programa).
    ///
    /// # Errors
    ///
    /// Uma mensagem para o usuário, se os argumentos estiverem errados.
    pub fn de_args(args: &[String]) -> Result<Config, String> {
        let mut cfg = Config {
            caminhos: Vec::new(),
            modo: Modo::default(),
            threads: 0,
            top: 10,
            por_arquivo: false,
            tempo: false,
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-m" | "--modo" => cfg.modo = valor(arg, args.next())?.parse()?,
                "-j" | "--threads" => cfg.threads = numero(arg, args.next())?,
                "-n" | "--top" => cfg.top = numero(arg, args.next())?,
                "--arquivos" => cfg.por_arquivo = true,
                "--tempo" => cfg.tempo = true,
                s if s.starts_with('-') && s != "-" => {
                    return Err(format!("opção desconhecida: {s}"));
                }
                caminho => cfg.caminhos.push(PathBuf::from(caminho)),
            }
        }
        if cfg.caminhos.is_empty() {
            return Err("faltou o CAMINHO".into());
        }
        Ok(cfg)
    }
}

fn valor<'a>(opcao: &str, v: Option<&'a String>) -> Result<&'a str, String> {
    v.map(String::as_str)
        .ok_or_else(|| format!("{opcao} precisa de um valor"))
}

fn numero(opcao: &str, v: Option<&String>) -> Result<usize, String> {
    let v = valor(opcao, v)?;
    v.parse()
        .map_err(|_| format!("{opcao}: número inválido: {v}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn padroes() {
        let c = Config::de_args(&args(&["src"])).unwrap();
        assert_eq!(c.modo, Modo::Rayon);
        assert_eq!(
            (c.threads, c.top, c.por_arquivo, c.tempo),
            (0, 10, false, false)
        );
        assert_eq!(c.caminhos, [PathBuf::from("src")]);
    }

    #[test]
    fn todas_as_opcoes() {
        let c = Config::de_args(&args(&[
            "-m",
            "canais",
            "-j",
            "4",
            "-n",
            "3",
            "--arquivos",
            "--tempo",
            "a",
            "b",
        ]))
        .unwrap();
        assert_eq!(c.modo, Modo::Canais);
        assert_eq!(
            (c.threads, c.top, c.por_arquivo, c.tempo),
            (4, 3, true, true)
        );
        assert_eq!(c.caminhos.len(), 2);
    }

    #[test]
    fn erros_de_uso() {
        for v in [
            &[][..],
            &["-m", "turbo", "x"],
            &["-j", "muitas", "x"],
            &["-n"],
            &["--rapido", "x"],
        ] {
            assert!(Config::de_args(&args(v)).is_err(), "{v:?}");
        }
    }
}
