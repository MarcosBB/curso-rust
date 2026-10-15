//! Lições 01 e 02: struct, `impl`, métodos e funções associadas.

#[derive(Debug, Clone, PartialEq)]
struct Rect {
    width: u32,
    height: u32,
}

impl Rect {
    /// Função associada (sem `self`): o "construtor" por convenção.
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Outra função associada.
    fn square(side: u32) -> Self {
        Self::new(side, side)
    }

    /// `&self`: só lê.
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rect) -> bool {
        self.width > other.width && self.height > other.height
    }

    /// `&mut self`: altera o valor no lugar.
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    /// `self`: consome o valor e devolve outro.
    fn into_square(self) -> Rect {
        let side = self.width.max(self.height);
        Rect::square(side)
    }
}

// Um segundo bloco `impl` para o mesmo tipo é permitido.
impl Rect {
    fn is_square(&self) -> bool {
        self.width == self.height
    }
}

fn main() {
    let mut r = Rect::new(3, 4);
    println!("área: {}", r.area());
    println!("cabe 2x2? {}", r.can_hold(&Rect::square(2)));
    r.scale(2);
    println!("{r:?}");
    println!("área (forma explícita): {}", Rect::area(&r));
    let q = r.into_square();
    println!("{q:?} é quadrado? {}", q.is_square());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_and_scale() {
        let mut r = Rect::new(3, 4);
        assert_eq!(r.area(), 12);
        r.scale(2);
        assert_eq!(r, Rect::new(6, 8));
    }

    #[test]
    fn into_square_uses_largest_side() {
        let q = Rect::new(2, 5).into_square();
        assert_eq!(q, Rect::square(5));
        assert!(q.is_square());
    }

    #[test]
    fn can_hold() {
        let big = Rect::new(10, 10);
        assert!(big.can_hold(&Rect::new(9, 9)));
        assert!(!big.can_hold(&Rect::new(10, 1)));
    }
}
