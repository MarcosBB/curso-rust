//! Solução do exercício 5 do Módulo 09: uma data com `Display` e `FromStr`
//! (Módulo 06) e a propriedade de ida e volta.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data {
    pub ano: u16,
    pub mes: u8,
    pub dia: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroData {
    /// O texto não está no formato `AAAA-MM-DD`.
    Formato,
    /// O formato está certo, mas o mês ou o dia não existem.
    Invalida,
}

impl Data {
    /// Cria uma data (simplificada: todo mês tem até 31 dias).
    ///
    /// # Errors
    ///
    /// [`ErroData::Invalida`] se o mês não for de 1 a 12 ou o dia de 1 a 31.
    pub fn nova(ano: u16, mes: u8, dia: u8) -> Result<Data, ErroData> {
        if !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
            return Err(ErroData::Invalida);
        }
        Ok(Data { ano, mes, dia })
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A versão com bug usava `{}` no mês: 2024-3-05.
        write!(f, "{:04}-{:02}-{:02}", self.ano, self.mes, self.dia)
    }
}

impl FromStr for Data {
    type Err = ErroData;

    /// Lê exatamente `AAAA-MM-DD`.
    fn from_str(s: &str) -> Result<Data, ErroData> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(ErroData::Formato);
        }
        let numero =
            |faixa: std::ops::Range<usize>| s[faixa].parse().map_err(|_| ErroData::Formato);
        let ano = numero(0..4)?;
        let mes = u8::try_from(numero(5..7)?).map_err(|_| ErroData::Formato)?;
        let dia = u8::try_from(numero(8..10)?).map_err(|_| ErroData::Formato)?;
        Data::nova(ano, mes, dia)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn le_uma_data() {
        assert_eq!("2026-10-24".parse(), Data::nova(2026, 10, 24));
    }

    #[test]
    fn formato_errado() {
        assert_eq!("24/10/2026".parse::<Data>(), Err(ErroData::Formato));
        assert_eq!("2026-13-01".parse::<Data>(), Err(ErroData::Invalida));
    }

    /// O caso mínimo que o proptest encontrou na versão com bug.
    #[test]
    fn mes_de_um_digito() {
        let data = Data::nova(1000, 1, 1).unwrap();
        assert_eq!(data.to_string(), "1000-01-01");
        assert_eq!(data.to_string().parse(), Ok(data));
    }

    fn uma_data() -> impl Strategy<Value = Data> {
        (1000u16..=9999, 1u8..=12, 1u8..=31).prop_map(|(ano, mes, dia)| Data { ano, mes, dia })
    }

    proptest! {
        #[test]
        fn ida_e_volta(data in uma_data()) {
            prop_assert_eq!(data.to_string().parse::<Data>(), Ok(data));
        }
    }
}
