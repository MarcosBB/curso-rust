//! Lições 03 e 04: tamanho de enums e a otimização de nicho de `Option`.
//! Os números valem para uma plataforma de 64 bits.

use std::mem::size_of;
use std::num::NonZeroU32;

#[allow(dead_code)]
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Unit,
}

#[allow(dead_code)]
enum Direction {
    North,
    East,
    South,
    West,
}

/// Imprime o tamanho do tipo `T`. O `<T>` é um parâmetro genérico (Módulo 06).
fn show<T>(name: &str) {
    println!("{name:<20} {:>2} bytes", size_of::<T>());
}

fn main() {
    show::<Shape>("Shape");
    show::<Direction>("Direction");
    println!();
    show::<u32>("u32");
    show::<Option<u32>>("Option<u32>");
    show::<u64>("u64");
    show::<Option<u64>>("Option<u64>");
    println!();
    show::<&i32>("&i32");
    show::<Option<&i32>>("Option<&i32>");
    show::<Box<i32>>("Box<i32>");
    show::<Option<Box<i32>>>("Option<Box<i32>>");
    show::<String>("String");
    show::<Option<String>>("Option<String>");
    show::<NonZeroU32>("NonZeroU32");
    show::<Option<NonZeroU32>>("Option<NonZeroU32>");
    show::<bool>("bool");
    show::<Option<bool>>("Option<bool>");
}
