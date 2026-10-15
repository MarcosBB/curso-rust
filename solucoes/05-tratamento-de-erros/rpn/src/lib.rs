//! Calculadora em notação polonesa reversa (RPN): `"3 4 + 2 *"` vale 14.

use std::error::Error;
use std::fmt;

/// `pos` é a posição do token (a partir de 0) em que o erro aconteceu.
#[derive(Debug, PartialEq)]
pub enum EvalError {
    /// Um operador sem dois números antes dele.
    MissingOperand { pos: usize },
    /// Um token que não é número nem operador.
    BadToken { pos: usize, token: String },
    /// Divisão por zero.
    DivisionByZero { pos: usize },
    /// O resultado não cabe em um `i64`.
    Overflow { pos: usize },
    /// No fim, a pilha deveria ter exatamente um número.
    WrongStackSize { count: usize },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::MissingOperand { pos } => {
                write!(f, "posição {pos}: operador sem dois operandos")
            }
            EvalError::BadToken { pos, token } => {
                write!(f, "posição {pos}: token inválido `{token}`")
            }
            EvalError::DivisionByZero { pos } => write!(f, "posição {pos}: divisão por zero"),
            EvalError::Overflow { pos } => write!(f, "posição {pos}: overflow"),
            EvalError::WrongStackSize { count } => {
                write!(
                    f,
                    "a expressão deixou {count} valores na pilha (esperava 1)"
                )
            }
        }
    }
}

impl Error for EvalError {}

pub fn eval(expr: &str) -> Result<i64, EvalError> {
    let mut stack: Vec<i64> = Vec::new();
    for (pos, token) in expr.split_whitespace().enumerate() {
        match token {
            "+" | "-" | "*" | "/" => {
                let b = stack.pop().ok_or(EvalError::MissingOperand { pos })?;
                let a = stack.pop().ok_or(EvalError::MissingOperand { pos })?;
                let r = match token {
                    "+" => a.checked_add(b),
                    "-" => a.checked_sub(b),
                    "*" => a.checked_mul(b),
                    _ => {
                        if b == 0 {
                            return Err(EvalError::DivisionByZero { pos });
                        }
                        a.checked_div(b)
                    }
                };
                stack.push(r.ok_or(EvalError::Overflow { pos })?);
            }
            _ => {
                let n = token.parse().map_err(|_| EvalError::BadToken {
                    pos,
                    token: token.to_string(),
                })?;
                stack.push(n);
            }
        }
    }
    if stack.len() != 1 {
        return Err(EvalError::WrongStackSize { count: stack.len() });
    }
    Ok(stack[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressoes_validas() {
        assert_eq!(eval("3 4 +"), Ok(7));
        assert_eq!(eval("3 4 + 2 *"), Ok(14));
        assert_eq!(eval("10 3 -"), Ok(7));
        assert_eq!(eval("7 2 /"), Ok(3));
        assert_eq!(eval("-5"), Ok(-5));
    }

    #[test]
    fn falta_operando() {
        assert_eq!(eval("1 +"), Err(EvalError::MissingOperand { pos: 1 }));
    }

    #[test]
    fn token_invalido() {
        assert_eq!(
            eval("1 x +"),
            Err(EvalError::BadToken {
                pos: 1,
                token: "x".to_string()
            })
        );
    }

    #[test]
    fn divisao_por_zero() {
        assert_eq!(eval("4 0 /"), Err(EvalError::DivisionByZero { pos: 2 }));
    }

    #[test]
    fn overflow() {
        assert_eq!(
            eval("9223372036854775807 1 +"),
            Err(EvalError::Overflow { pos: 2 })
        );
    }

    #[test]
    fn pilha_errada_no_fim() {
        assert_eq!(eval("1 2"), Err(EvalError::WrongStackSize { count: 2 }));
        assert_eq!(eval(""), Err(EvalError::WrongStackSize { count: 0 }));
    }

    #[test]
    fn mensagens() {
        assert_eq!(
            eval("4 0 /").unwrap_err().to_string(),
            "posição 2: divisão por zero"
        );
        assert_eq!(
            eval("1 2").unwrap_err().to_string(),
            "a expressão deixou 2 valores na pilha (esperava 1)"
        );
    }
}
