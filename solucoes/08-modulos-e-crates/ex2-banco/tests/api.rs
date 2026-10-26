use banco::auditoria::extrato;
use banco::conta::Conta;

#[test]
fn deposito_e_saque_com_tarifa() {
    let mut c = Conta::abrir("Ana");
    c.depositar(1000);
    assert_eq!(c.sacar(500), Ok(()));
    assert_eq!(c.saldo(), 1000 - 500 - 7);
    assert_eq!(c.titular(), "Ana");
}

#[test]
fn saque_sem_saldo_falha_e_nao_muda_nada() {
    let mut c = Conta::abrir("Bruno");
    c.depositar(100);
    assert!(c.sacar(100).is_err());
    assert_eq!(c.saldo(), 100);
}

#[test]
fn extrato_lista_os_lancamentos() {
    let mut c = Conta::abrir("Carla");
    c.depositar(300);
    c.sacar(100).unwrap();
    assert_eq!(extrato(&c), "Carla\n+300\n-103\nsaldo: 197\n");
}
