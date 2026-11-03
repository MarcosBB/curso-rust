//! Exercício 4: uma macro que gera um enum e um `impl` com a lista e os nomes.

/// Gera o enum como escrito e mais:
/// - `TODOS`: todas as variantes, na ordem;
/// - `nome(&self)`: o nome da variante;
/// - `de_nome(&str)`: o caminho inverso.
#[macro_export]
macro_rules! enum_com_nomes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $nome:ident {
            $($variante:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $nome {
            $($variante),+
        }

        impl $nome {
            /// Todas as variantes, na ordem da declaração.
            pub const TODOS: &'static [$nome] = &[$($nome::$variante),+];

            /// O nome da variante, como foi escrito.
            pub fn nome(&self) -> &'static str {
                match self {
                    $($nome::$variante => stringify!($variante)),+
                }
            }

            /// A variante com esse nome, se existir.
            pub fn de_nome(texto: &str) -> ::std::option::Option<$nome> {
                match texto {
                    $(stringify!($variante) => ::std::option::Option::Some($nome::$variante),)+
                    _ => ::std::option::Option::None,
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    enum_com_nomes! {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Dia {
            Segunda,
            Terca,
            Quarta,
        }
    }

    enum_com_nomes! {
        #[derive(Debug, PartialEq)]
        enum Nivel { Baixo, Alto }
    }

    #[test]
    fn todos_na_ordem() {
        assert_eq!(Dia::TODOS, [Dia::Segunda, Dia::Terca, Dia::Quarta]);
        assert_eq!(Nivel::TODOS.len(), 2);
    }

    #[test]
    fn nome_e_de_nome() {
        assert_eq!(Dia::Terca.nome(), "Terca");
        assert_eq!(Dia::de_nome("Quarta"), Some(Dia::Quarta));
        assert_eq!(Dia::de_nome("Domingo"), None);
        assert_eq!(Nivel::de_nome("Alto"), Some(Nivel::Alto));
        assert_eq!(Nivel::Baixo.nome(), "Baixo");
    }

    #[test]
    fn ida_e_volta() {
        for d in Dia::TODOS {
            assert_eq!(Dia::de_nome(d.nome()), Some(*d));
        }
    }
}
