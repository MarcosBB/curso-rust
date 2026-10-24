//! A busca em si: recebe o texto já lido e devolve as linhas que casam.

/// Uma linha que casou com o padrão.
///
/// `line` aponta para dentro do texto original (`contents`), sem cópia.
/// Por isso a struct carrega o lifetime `'a`: ela não pode viver mais que o texto.
#[derive(Debug, PartialEq, Eq)]
pub struct Match<'a> {
    /// Número da linha, começando em 1 (como no `grep -n`).
    pub line_no: usize,
    pub line: &'a str,
}

/// Opções que mudam o que conta como "casar".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// `-i`: ignora maiúsculas e minúsculas.
    pub ignore_case: bool,
    /// `-v`: devolve as linhas que **não** casam.
    pub invert: bool,
}

/// Devolve as linhas de `contents` que contêm `pattern`.
///
/// O retorno só depende do lifetime de `contents`: o padrão pode ser
/// descartado logo depois da chamada.
///
/// Um padrão vazio casa com todas as linhas (`"abc".contains("")` é `true`),
/// o mesmo comportamento do `grep ''`.
pub fn search<'a>(pattern: &str, contents: &'a str, opts: SearchOptions) -> Vec<Match<'a>> {
    // Só precisamos da versão minúscula do padrão com `-i`, mas calcular
    // uma vez aqui é mais simples do que calcular a cada linha.
    let pattern_lower = pattern.to_lowercase();
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let is_hit = if opts.ignore_case {
                line.to_lowercase().contains(&pattern_lower)
            } else {
                line.contains(pattern)
            };
            // Com `invert`, ficam justamente as linhas que não casaram.
            is_hit != opts.invert
        })
        .map(|(i, line)| Match {
            line_no: i + 1,
            line,
        })
        .collect()
}

/// Versão preguiçosa de `search` (parte "para ir além" do exercício).
///
/// Nada é calculado até alguém puxar: `search_iter(..).next()` para na
/// primeira linha que casa. O iterador guarda `pattern` (a closure do
/// `filter` o usa a cada linha), então ele precisa viver tanto quanto o
/// iterador: daí o mesmo `'a` nos dois parâmetros. Em `search`, o
/// resultado só depende de `contents`.
pub fn search_iter<'a>(
    pattern: &'a str,
    contents: &'a str,
    opts: SearchOptions,
) -> impl Iterator<Item = Match<'a>> {
    let pattern_lower = pattern.to_lowercase();
    contents
        .lines()
        .enumerate()
        .filter(move |(_, line)| {
            let is_hit = if opts.ignore_case {
                line.to_lowercase().contains(&pattern_lower)
            } else {
                line.contains(pattern)
            };
            is_hit != opts.invert
        })
        .map(|(i, line)| Match {
            line_no: i + 1,
            line,
        })
}

/// Texto usado pelos testes das etapas 1 e 2 (o mesmo do Rust Book).
#[cfg(test)]
const POEM: &str = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.
Trust me.";

#[cfg(test)]
mod step1_tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let matches = search("duct", POEM, SearchOptions::default());
        assert_eq!(
            matches,
            vec![Match {
                line_no: 2,
                line: "safe, fast, productive."
            }]
        );
    }

    #[test]
    fn line_numbers_start_at_one() {
        let matches = search("Rust", POEM, SearchOptions::default());
        assert_eq!(matches[0].line_no, 1);
    }

    #[test]
    fn empty_pattern_matches_every_line() {
        let matches = search("", POEM, SearchOptions::default());
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn results_borrow_from_contents() {
        let matches = search("three", POEM, SearchOptions::default());
        // A linha devolvida é uma fatia de POEM, e não uma cópia:
        // o ponteiro aponta para dentro do texto original.
        let start = POEM.find("Pick").unwrap();
        assert!(std::ptr::eq(
            matches[0].line.as_ptr(),
            POEM[start..].as_ptr()
        ));
    }

    #[test]
    fn pattern_can_be_dropped_before_results() {
        let matches;
        {
            let pattern = String::from("tape");
            matches = search(&pattern, POEM, SearchOptions::default());
        } // `pattern` é destruído aqui, e `matches` continua válido
        assert_eq!(matches[0].line, "Duct tape.");
    }
}

#[cfg(test)]
mod step2_tests {
    use super::*;

    #[test]
    fn case_insensitive() {
        let opts = SearchOptions {
            ignore_case: true,
            ..Default::default()
        };
        let lines: Vec<&str> = search("rUsT", POEM, opts).iter().map(|m| m.line).collect();
        assert_eq!(lines, ["Rust:", "Trust me."]);
    }

    #[test]
    fn invert() {
        let opts = SearchOptions {
            invert: true,
            ..Default::default()
        };
        let matches = search("e", POEM, opts);
        assert_eq!(
            matches,
            vec![Match {
                line_no: 1,
                line: "Rust:"
            }]
        );
    }

    #[test]
    fn invert_and_ignore_case() {
        let opts = SearchOptions {
            ignore_case: true,
            invert: true,
        };
        // Sobram as linhas sem "t" nem "T".
        assert_eq!(search("T", POEM, opts).len(), 0);
        assert_eq!(search("DUCT", POEM, opts).len(), 3);
    }
}

#[cfg(test)]
mod m07_tests {
    use super::*;

    #[test]
    fn lazy_version_matches_eager_one() {
        for opts in [
            SearchOptions::default(),
            SearchOptions {
                ignore_case: true,
                invert: false,
            },
            SearchOptions {
                ignore_case: true,
                invert: true,
            },
        ] {
            let eager = search("t", POEM, opts);
            let lazy: Vec<Match> = search_iter("t", POEM, opts).collect();
            assert_eq!(eager, lazy);
        }
    }

    #[test]
    fn lazy_version_stops_early() {
        let first = search_iter("e", POEM, SearchOptions::default()).next();
        assert_eq!(first.map(|m| m.line_no), Some(2));
    }
}
