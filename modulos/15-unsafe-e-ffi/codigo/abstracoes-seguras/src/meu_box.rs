//! Um `Box` mínimo: posse de um valor no heap através de um ponteiro cru.

use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Dono de um `T` no heap.
///
/// Invariante: `ptr` veio de `Box::into_raw` e é liberado só no `Drop`.
pub struct MeuBox<T> {
    ptr: NonNull<T>,
}

impl<T> MeuBox<T> {
    pub fn new(valor: T) -> Self {
        let cru = Box::into_raw(Box::new(valor));
        // SAFETY: `Box::into_raw` nunca devolve ponteiro nulo.
        let ptr = unsafe { NonNull::new_unchecked(cru) };
        MeuBox { ptr }
    }
}

impl<T> Deref for MeuBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: pela invariante, `ptr` aponta para um `T` vivo, e o `&T`
        // devolvido empresta `self`, então não vive mais que o `MeuBox`.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for MeuBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: como em `deref`; o `&mut self` garante acesso exclusivo.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for MeuBox<T> {
    fn drop(&mut self) {
        // SAFETY: `ptr` veio de `Box::into_raw` e o `drop` roda uma vez só.
        unsafe { drop(Box::from_raw(self.ptr.as_ptr())) };
    }
}

// `NonNull<T>` não é `Send` nem `Sync`, então o compilador não deriva esses
// traits para `MeuBox`. Como `MeuBox<T>` é dono exclusivo do `T` (igual a
// `Box<T>`), enviá-lo para outra thread é tão seguro quanto enviar o `T`.
// SAFETY: `MeuBox<T>` tem posse exclusiva do `T`; nada mais aponta para ele.
unsafe impl<T: Send> Send for MeuBox<T> {}
// SAFETY: `&MeuBox<T>` só dá acesso a `&T`, então é seguro se `T: Sync`.
unsafe impl<T: Sync> Sync for MeuBox<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usa_como_box() {
        let mut b = MeuBox::new(String::from("oi"));
        b.push_str(", mundo");
        assert_eq!(b.len(), 9);
        assert_eq!(&*b, "oi, mundo");
    }

    #[test]
    fn envia_para_outra_thread() {
        let b = MeuBox::new(vec![1, 2, 3]);
        let soma = std::thread::spawn(move || b.iter().sum::<i32>())
            .join()
            .unwrap();
        assert_eq!(soma, 6);
    }
}
