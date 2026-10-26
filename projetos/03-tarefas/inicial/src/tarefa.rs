use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Prioridade de uma tarefa. A ordem das variantes é a ordem de `Ord`:
/// `Baixa < Media < Alta`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
// Etapa 2: falta um atributo aqui (os nomes no JSON são minúsculos).
pub enum Prioridade {
    Baixa,
    #[default]
    Media,
    Alta,
}

impl FromStr for Prioridade {
    type Err = String;

    /// Aceita o nome inteiro ou a inicial, com ou sem acento, sem diferenciar
    /// maiúsculas: `alta`, `A`, `média`, `m`, `baixa`, `b`...
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Etapa 1.
        todo!()
    }
}

impl fmt::Display for Prioridade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Etapa 1: "alta", "media" ou "baixa", respeitando largura (`{:<5}`).
        todo!()
    }
}

/// Uma tarefa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tarefa {
    pub id: u32,
    pub titulo: String,
    pub prioridade: Prioridade,
    pub feita: bool,
    /// Segundos desde 1970-01-01 (UNIX).
    pub criada_em: u64,
    /// Não aparece no JSON quando vazio, e arquivos sem o campo continuam
    /// carregando (ficam com a lista vazia).
    // Etapa 2: falta um atributo aqui.
    pub etiquetas: Vec<String>,
}

impl fmt::Display for Tarefa {
    /// `[x]   2  media  Revisar o Projeto 2`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Etapa 1.
        todo!()
    }
}

#[cfg(test)]
fn exemplo() -> Tarefa {
    Tarefa {
        id: 1,
        titulo: "Estudar o M09".into(),
        prioridade: Prioridade::Alta,
        feita: false,
        criada_em: 0,
        etiquetas: vec![],
    }
}

#[cfg(test)]
mod step1_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::alta("alta", Prioridade::Alta)]
    #[case::inicial_maiuscula("A", Prioridade::Alta)]
    #[case::com_acento("média", Prioridade::Media)]
    #[case::maiusculas_com_acento("MÉDIA", Prioridade::Media)]
    #[case::inicial("b", Prioridade::Baixa)]
    fn prioridade_valida(#[case] texto: &str, #[case] esperada: Prioridade) {
        assert_eq!(texto.parse::<Prioridade>(), Ok(esperada));
    }

    #[test]
    fn prioridade_invalida() {
        assert_eq!(
            "urgente".parse::<Prioridade>(),
            Err("prioridade inválida: urgente (use alta, media ou baixa)".to_string())
        );
    }

    #[test]
    fn ordem_e_padrao() {
        assert!(Prioridade::Baixa < Prioridade::Media);
        assert!(Prioridade::Media < Prioridade::Alta);
        assert_eq!(Prioridade::default(), Prioridade::Media);
    }

    #[test]
    fn display_respeita_largura() {
        assert_eq!(format!("[{:<5}]", Prioridade::Alta), "[alta ]");
        assert_eq!(Prioridade::Baixa.to_string(), "baixa");
    }

    #[test]
    fn formato_da_linha() {
        let mut t = exemplo();
        assert_eq!(t.to_string(), "[ ]   1  alta   Estudar o M09");
        t.feita = true;
        t.id = 12;
        t.prioridade = Prioridade::Media;
        assert_eq!(t.to_string(), "[x]  12  media  Estudar o M09");
    }
}

#[cfg(test)]
mod step2_tests {
    use super::*;

    #[test]
    fn json_da_tarefa() {
        let json = serde_json::to_string(&exemplo()).unwrap();
        insta::assert_snapshot!(json, @r#"{"id":1,"titulo":"Estudar o M09","prioridade":"alta","feita":false,"criada_em":0}"#);
    }

    #[test]
    fn etiquetas_aparecem_quando_existem() {
        let t = Tarefa {
            etiquetas: vec!["rust".into()],
            ..exemplo()
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.ends_with(r#","etiquetas":["rust"]}"#), "{json}");
    }

    #[test]
    fn arquivo_antigo_sem_etiquetas_ainda_carrega() {
        let json = r#"{"id":2,"titulo":"x","prioridade":"baixa","feita":true,"criada_em":5}"#;
        let t: Tarefa = serde_json::from_str(json).unwrap();
        assert_eq!(t.prioridade, Prioridade::Baixa);
        assert!(t.etiquetas.is_empty());
    }

    #[test]
    fn prioridade_desconhecida_no_json() {
        let json = r#"{"id":2,"titulo":"x","prioridade":"urgente","feita":true,"criada_em":5}"#;
        let erro = serde_json::from_str::<Tarefa>(json).unwrap_err();
        assert_eq!(
            erro.to_string(),
            "unknown variant `urgente`, expected one of `baixa`, `media`, `alta` at line 1 column 43"
        );
    }
}
