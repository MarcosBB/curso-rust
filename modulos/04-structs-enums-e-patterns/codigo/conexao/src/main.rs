//! Lição 08: estados inválidos irrepresentáveis.
//! Um enum de estados no lugar de flags, e um newtype com construtor validado.

mod net {
    /// Porta não privilegiada (>= 1024). O campo é privado: fora deste
    /// módulo, a única forma de criar uma `Port` é `Port::new`.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Port(u16);

    impl Port {
        pub fn new(value: u16) -> Option<Port> {
            if value >= 1024 {
                Some(Port(value))
            } else {
                None
            }
        }

        pub fn get(self) -> u16 {
            self.0
        }
    }
}

use net::Port;

/// Cada estado carrega só os dados que fazem sentido nele.
#[derive(Debug, Clone, PartialEq)]
enum Conn {
    Disconnected,
    Connecting { port: Port, attempt: u8 },
    Connected { session_id: u64 },
    Failed { reason: String },
}

impl Conn {
    fn connect(self, port: Port) -> Conn {
        match self {
            Conn::Disconnected | Conn::Failed { .. } => Conn::Connecting { port, attempt: 1 },
            other => other,
        }
    }

    fn on_success(self, session_id: u64) -> Conn {
        match self {
            Conn::Connecting { .. } => Conn::Connected { session_id },
            other => other,
        }
    }

    fn on_timeout(self) -> Conn {
        match self {
            Conn::Connecting { port, attempt } if attempt < 3 => Conn::Connecting {
                port,
                attempt: attempt + 1,
            },
            Conn::Connecting { .. } => Conn::Failed {
                reason: String::from("tempo esgotado"),
            },
            other => other,
        }
    }

    fn disconnect(self) -> Conn {
        Conn::Disconnected
    }

    fn status(&self) -> String {
        match self {
            Conn::Disconnected => String::from("desconectado"),
            Conn::Connecting { port, attempt } => {
                format!("conectando na porta {} (tentativa {attempt})", port.get())
            }
            Conn::Connected { session_id } => format!("conectado, sessão {session_id}"),
            Conn::Failed { reason } => format!("falhou: {reason}"),
        }
    }
}

fn main() {
    let port = Port::new(8080).expect("8080 é uma porta válida");
    println!("{:?} {:?}", Port::new(80), Port::new(5432));

    let mut c = Conn::Disconnected.connect(port);
    println!("{}", c.status());
    c = c.on_timeout();
    println!("{}", c.status());
    c = c.on_success(42);
    println!("{}", c.status());
    c = c.disconnect();
    println!("{}", c.status());

    let mut d = Conn::Disconnected.connect(port);
    for _ in 0..3 {
        d = d.on_timeout();
    }
    println!("{}", d.status());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port() -> Port {
        Port::new(8080).unwrap()
    }

    #[test]
    fn port_validation() {
        assert_eq!(Port::new(1023), None);
        assert_eq!(Port::new(1024).map(Port::get), Some(1024));
    }

    #[test]
    fn happy_path() {
        let c = Conn::Disconnected.connect(port()).on_success(7);
        assert_eq!(c, Conn::Connected { session_id: 7 });
    }

    #[test]
    fn gives_up_after_three_attempts() {
        let c = Conn::Disconnected
            .connect(port())
            .on_timeout()
            .on_timeout()
            .on_timeout();
        assert!(matches!(c, Conn::Failed { .. }));
    }

    #[test]
    fn success_without_connecting_is_ignored() {
        assert_eq!(Conn::Disconnected.on_success(1), Conn::Disconnected);
    }
}
