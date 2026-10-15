fn area_retangulo(largura: f64, altura: f64) -> f64 {
    largura * altura
}

fn main() {
    let largura = 3.0;
    let altura = 4.5;
    let area = area_retangulo(largura, altura);
    if area > 10.0 {
        println!("área grande: {area}");
    } else {
        println!("área pequena: {area}");
    }
}
