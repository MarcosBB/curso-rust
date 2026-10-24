//! Closures na prática (lições 04 e 05): um bound para cada trait,
//! closures devolvidas com `impl Fn`/`impl FnMut` e uma lista de
//! regras com `Box<dyn Fn>`.

/// Chama `f` até dar certo, no máximo `tentativas` vezes.
///
/// Pede `FnMut`: chama várias vezes, uma de cada vez, e aceita closures
/// que alteram estado (um contador de tentativas, por exemplo).
pub fn tentar_de_novo<T, E>(tentativas: u32, mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    let mut ultimo = f();
    for _ in 1..tentativas {
        if ultimo.is_ok() {
            break;
        }
        ultimo = f();
    }
    ultimo
}

/// Devolve o valor, ou calcula o padrão só se precisar.
///
/// Pede `FnOnce`: chama no máximo uma vez, e assim aceita até closures
/// que movem algo para fora (como `Option::unwrap_or_else`).
pub fn ou_calcule<T>(valor: Option<T>, padrao: impl FnOnce() -> T) -> T {
    match valor {
        Some(v) => v,
        None => padrao(),
    }
}

/// Uma fábrica de contadores: cada chamada devolve o próximo número.
///
/// A closure é `FnMut` (altera `atual`) e precisa de `move`, porque
/// `atual` morreria quando `contador` retornasse (E0373).
pub fn contador(inicio: u32) -> impl FnMut() -> u32 {
    let mut atual = inicio;
    move || {
        let v = atual;
        atual += 1;
        v
    }
}

/// Composição: `compor(f, g)(x) == g(f(x))`.
pub fn compor<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

/// Uma regra recebe o texto e devolve uma mensagem de erro, se houver.
type Regra = Box<dyn Fn(&str) -> Option<String>>;

/// Uma lista de regras de tipos diferentes: por isso `Box<dyn Fn>`,
/// e não um genérico (cada closure tem um tipo próprio).
#[derive(Default)]
pub struct Validador {
    regras: Vec<Regra>,
}

impl Validador {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta uma regra. O `'static` diz que a closure não pode
    /// guardar empréstimos de variáveis locais: use `move`.
    pub fn regra(mut self, r: impl Fn(&str) -> Option<String> + 'static) -> Self {
        self.regras.push(Box::new(r));
        self
    }

    pub fn validar(&self, texto: &str) -> Vec<String> {
        self.regras.iter().filter_map(|r| r(texto)).collect()
    }
}

/// Um validador de senha montado com closures que capturam parâmetros.
pub fn validador_de_senha(tamanho_minimo: usize, proibidas: Vec<String>) -> Validador {
    Validador::new()
        .regra(move |s| {
            (s.chars().count() < tamanho_minimo)
                .then(|| format!("precisa de pelo menos {tamanho_minimo} caracteres"))
        })
        .regra(|s| {
            (!s.chars().any(|c| c.is_ascii_digit())).then(|| "precisa de um dígito".to_string())
        })
        .regra(move |s| {
            proibidas
                .iter()
                .any(|p| s.contains(p.as_str()))
                .then(|| "contém uma palavra proibida".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tentar_de_novo_conta_tentativas() {
        let mut chamadas = 0;
        let r: Result<u32, &str> = tentar_de_novo(5, || {
            chamadas += 1;
            if chamadas < 3 {
                Err("ainda não")
            } else {
                Ok(chamadas)
            }
        });
        assert_eq!(r, Ok(3));
        assert_eq!(chamadas, 3);

        let falha: Result<(), &str> = tentar_de_novo(2, || Err("nunca"));
        assert_eq!(falha, Err("nunca"));
    }

    #[test]
    fn ou_calcule_so_chama_se_precisar() {
        let nome = String::from("padrão");
        // A closure move `nome` para fora: é só FnOnce, e serve.
        assert_eq!(ou_calcule(None, || nome), "padrão");

        let mut chamou = false;
        assert_eq!(
            ou_calcule(Some(1), || {
                chamou = true;
                0
            }),
            1
        );
        assert!(!chamou);
    }

    #[test]
    fn contadores_sao_independentes() {
        let mut a = contador(10);
        let mut b = contador(0);
        assert_eq!((a(), a(), b(), a()), (10, 11, 0, 12));
    }

    #[test]
    fn compor_aplica_na_ordem() {
        let dobra_e_descreve = compor(|x: i32| x * 2, |y: i32| format!("<{y}>"));
        assert_eq!(dobra_e_descreve(21), "<42>");
    }

    #[test]
    fn validador() {
        let v = validador_de_senha(8, vec!["senha".to_string()]);
        assert!(v.validar("abc12345").is_empty());
        assert_eq!(
            v.validar("senha"),
            [
                "precisa de pelo menos 8 caracteres",
                "precisa de um dígito",
                "contém uma palavra proibida"
            ]
        );
    }
}
