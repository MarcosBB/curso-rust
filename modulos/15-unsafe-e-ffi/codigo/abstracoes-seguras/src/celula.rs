//! Uma `Cell` simplificada, feita com `UnsafeCell`.

use std::cell::UnsafeCell;

/// Valor `Copy` que pode ser trocado através de `&self`.
///
/// `UnsafeCell<T>` não é `Sync`, então `Celula<T>` também não é: o compilador
/// impede compartilhar uma `Celula` entre threads, e é isso que torna
/// `get`/`set` seguros.
pub struct Celula<T: Copy> {
    valor: UnsafeCell<T>,
}

impl<T: Copy> Celula<T> {
    pub fn new(valor: T) -> Self {
        Celula {
            valor: UnsafeCell::new(valor),
        }
    }

    pub fn get(&self) -> T {
        // SAFETY: `Celula` não é `Sync`, então só esta thread acessa o valor.
        // Nenhuma referência para o conteúdo sai deste método: lemos uma cópia.
        unsafe { *self.valor.get() }
    }

    pub fn set(&self, novo: T) {
        // SAFETY: como em `get`. Nenhum `&T` para o conteúdo existe agora,
        // porque a API nunca entrega um; então escrever não invalida ninguém.
        unsafe { *self.valor.get() = novo };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn troca_por_referencia_compartilhada() {
        let c = Celula::new(1);
        let a = &c;
        let b = &c;
        a.set(2);
        b.set(b.get() + 1);
        assert_eq!(c.get(), 3);
    }
}
