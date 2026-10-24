use conversoes::{Percentual, saudar};

fn main() {
    for texto in ["75%", "150%", "abc%", "42"] {
        match texto.parse::<Percentual>() {
            Ok(p) => println!("{texto:>5} -> {p} = {}", f64::from(p)),
            Err(e) => println!("{texto:>5} -> erro: {e}"),
        }
    }
    println!("{}", saudar("Ana"));
}
