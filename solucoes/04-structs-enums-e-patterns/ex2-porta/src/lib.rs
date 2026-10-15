pub mod net {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Port(u16);

    impl Port {
        /// Aceita só portas não privilegiadas (>= 1024).
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

        /// Faixa dinâmica/efêmera da IANA: 49152 a 65535.
        pub fn is_ephemeral(self) -> bool {
            self.0 >= 49152
        }
    }
}

#[cfg(test)]
mod tests {
    use super::net::Port;

    #[test]
    fn rejects_privileged_ports() {
        assert_eq!(Port::new(0), None);
        assert_eq!(Port::new(80), None);
        assert_eq!(Port::new(1023), None);
    }

    #[test]
    fn accepts_the_rest() {
        assert_eq!(Port::new(1024).map(Port::get), Some(1024));
        assert_eq!(Port::new(65535).map(Port::get), Some(65535));
    }

    #[test]
    fn ephemeral() {
        assert!(!Port::new(8080).unwrap().is_ephemeral());
        assert!(Port::new(49152).unwrap().is_ephemeral());
    }
}
