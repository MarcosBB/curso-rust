use formas::{Area, Circulo, Retangulo, area_total};

#[test]
fn areas_individuais() {
    assert_eq!(
        Retangulo {
            largura: 2.0,
            altura: 5.0
        }
        .area(),
        10.0
    );
    let c = Circulo { raio: 2.0 };
    assert!((c.area() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
}

#[test]
fn soma_formas_diferentes() {
    let c = Circulo { raio: 1.0 };
    let r = Retangulo {
        largura: 1.0,
        altura: 1.0,
    };
    let total = area_total(&[&c, &r]);
    assert!((total - (std::f64::consts::PI + 1.0)).abs() < 1e-12);
}
