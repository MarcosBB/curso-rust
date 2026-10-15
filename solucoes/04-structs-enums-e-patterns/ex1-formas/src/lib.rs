use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Square { side: f64 },
    Rect { w: f64, h: f64 },
    Triangle { a: f64, b: f64, c: f64 },
}

pub fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => PI * r * r,
        Shape::Square { side } => side * side,
        Shape::Rect { w, h } => w * h,
        // Fórmula de Heron.
        Shape::Triangle { a, b, c } => {
            let p = (a + b + c) / 2.0;
            (p * (p - a) * (p - b) * (p - c)).sqrt()
        }
    }
}

pub fn perimeter(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => 2.0 * PI * r,
        Shape::Square { side } => 4.0 * side,
        Shape::Rect { w, h } => 2.0 * (w + h),
        Shape::Triangle { a, b, c } => a + b + c,
    }
}

pub fn name(s: &Shape) -> &'static str {
    match s {
        Shape::Circle { .. } => "círculo",
        Shape::Square { .. } => "quadrado",
        Shape::Rect { .. } => "retângulo",
        Shape::Triangle { .. } => "triângulo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn circle() {
        let s = Shape::Circle { r: 1.0 };
        assert!(close(area(&s), PI));
        assert!(close(perimeter(&s), 2.0 * PI));
        assert_eq!(name(&s), "círculo");
    }

    #[test]
    fn square_and_rect() {
        assert!(close(area(&Shape::Square { side: 3.0 }), 9.0));
        assert!(close(perimeter(&Shape::Rect { w: 2.0, h: 5.0 }), 14.0));
        assert_eq!(name(&Shape::Rect { w: 2.0, h: 5.0 }), "retângulo");
    }

    #[test]
    fn triangle() {
        let t = Shape::Triangle {
            a: 3.0,
            b: 4.0,
            c: 5.0,
        };
        assert!(close(area(&t), 6.0));
        assert!(close(perimeter(&t), 12.0));
        assert_eq!(name(&t), "triângulo");
    }
}
