use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub name: String,
    pub max_users: u32,
}

/// Lê um arquivo com duas linhas: o nome do serviço e o número máximo de usuários.
pub fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let name = lines
        .next()
        .ok_or("faltou a linha do nome")?
        .trim()
        .to_string();
    let max_users = lines
        .next()
        .ok_or("faltou a linha de max_users")?
        .trim()
        .parse()?;
    Ok(Config { name, max_users })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    /// Cria um arquivo temporário com um nome único para este teste.
    fn temp_file(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("m05-ex1-{}-{name}", std::process::id()));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn arquivo_valido() {
        let p = temp_file("ok.txt", "loja\n42\n");
        let config = read_config(p.to_str().unwrap()).unwrap();
        assert_eq!(
            config,
            Config {
                name: "loja".to_string(),
                max_users: 42
            }
        );
    }

    #[test]
    fn arquivo_inexistente() {
        let e = read_config("/caminho/que/nao/existe.txt").unwrap_err();
        let io_err = e
            .downcast_ref::<io::Error>()
            .expect("deveria ser io::Error");
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn numero_invalido() {
        let p = temp_file("num.txt", "loja\nmuitos\n");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "invalid digit found in string");
    }

    #[test]
    fn linha_faltando() {
        let p = temp_file("falta.txt", "loja\n");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "faltou a linha de max_users");

        let p = temp_file("vazio.txt", "");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "faltou a linha do nome");
    }
}
