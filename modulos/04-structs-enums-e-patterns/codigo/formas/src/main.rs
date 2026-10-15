//! Lições 03 e 05: enum com dados, `impl` em enum e `match` exaustivo.

use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Unit,
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { r } => PI * r * r,
            Shape::Rect { w, h } => w * h,
            Shape::Unit => 0.0,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Shape::Circle { .. } => "círculo",
            Shape::Rect { w, h } if w == h => "quadrado",
            Shape::Rect { .. } => "retângulo",
            Shape::Unit => "ponto",
        }
    }
}

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn describe(m: &Message) -> String {
    match m {
        Message::Quit => String::from("sair"),
        Message::Move { x, y } => format!("mover para ({x}, {y})"),
        Message::Write(text) => format!("escrever {text:?}"),
        Message::ChangeColor(r, g, b) => format!("cor #{r:02x}{g:02x}{b:02x}"),
    }
}

fn main() {
    let shapes = [
        Shape::Circle { r: 1.0 },
        Shape::Rect { w: 2.0, h: 3.0 },
        Shape::Rect { w: 2.0, h: 2.0 },
        Shape::Unit,
    ];
    for s in &shapes {
        println!("{} {s:?} -> área {:.2}", s.name(), s.area());
    }

    let msgs = [
        Message::Move { x: 3, y: -1 },
        Message::Write(String::from("oi")),
        Message::ChangeColor(255, 128, 0),
        Message::Quit,
    ];
    for m in &msgs {
        println!("{}", describe(m));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn areas() {
        assert_eq!(Shape::Rect { w: 2.0, h: 3.0 }.area(), 6.0);
        assert_eq!(Shape::Unit.area(), 0.0);
        assert!((Shape::Circle { r: 1.0 }.area() - PI).abs() < 1e-12);
    }

    #[test]
    fn names() {
        assert_eq!(Shape::Rect { w: 2.0, h: 2.0 }.name(), "quadrado");
        assert_eq!(Shape::Rect { w: 2.0, h: 1.0 }.name(), "retângulo");
    }

    #[test]
    fn messages() {
        assert_eq!(describe(&Message::ChangeColor(255, 0, 16)), "cor #ff0010");
        assert_eq!(describe(&Message::Quit), "sair");
    }
}
