#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub price_cents: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Cart { items: Vec<Item> },
    Paid { items: Vec<Item>, total_cents: u64 },
    Shipped { tracking: String },
    Cancelled { reason: String },
}

impl Order {
    pub fn new() -> Order {
        Order::Cart { items: Vec::new() }
    }

    /// Só dá para adicionar itens no carrinho.
    pub fn add_item(&mut self, item: Item) -> bool {
        match self {
            Order::Cart { items } => {
                items.push(item);
                true
            }
            _ => false,
        }
    }

    /// Carrinho com itens vira `Paid`. Qualquer outro caso fica como está.
    pub fn pay(self) -> Order {
        match self {
            Order::Cart { items } if !items.is_empty() => {
                let total_cents = items.iter().map(|i| i.price_cents).sum();
                Order::Paid { items, total_cents }
            }
            other => other,
        }
    }

    pub fn ship(self, tracking: &str) -> Order {
        match self {
            Order::Paid { .. } => Order::Shipped {
                tracking: tracking.to_string(),
            },
            other => other,
        }
    }

    /// Dá para cancelar antes do envio, nunca depois.
    pub fn cancel(self, reason: &str) -> Order {
        match self {
            Order::Cart { .. } | Order::Paid { .. } => Order::Cancelled {
                reason: reason.to_string(),
            },
            other => other,
        }
    }

    pub fn status(&self) -> &'static str {
        match self {
            Order::Cart { items } if items.is_empty() => "carrinho vazio",
            Order::Cart { .. } => "carrinho",
            Order::Paid { .. } => "pago",
            Order::Shipped { .. } => "enviado",
            Order::Cancelled { .. } => "cancelado",
        }
    }

    pub fn total_cents(&self) -> Option<u64> {
        match self {
            Order::Paid { total_cents, .. } => Some(*total_cents),
            _ => None,
        }
    }
}

// O clippy pede um `Default` quando existe um `new()` sem argumentos.
// Implementar traits é assunto do Módulo 06.
impl Default for Order {
    fn default() -> Self {
        Order::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, price_cents: u64) -> Item {
        Item {
            name: name.to_string(),
            price_cents,
        }
    }

    #[test]
    fn happy_path() {
        let mut o = Order::new();
        assert_eq!(o.status(), "carrinho vazio");
        assert!(o.add_item(item("livro", 5000)));
        assert!(o.add_item(item("caneta", 350)));
        assert_eq!(o.status(), "carrinho");

        let o = o.pay();
        assert_eq!(o.status(), "pago");
        assert_eq!(o.total_cents(), Some(5350));

        let o = o.ship("BR123");
        assert_eq!(
            o,
            Order::Shipped {
                tracking: String::from("BR123")
            }
        );
    }

    #[test]
    fn empty_cart_cannot_be_paid() {
        let o = Order::new().pay();
        assert_eq!(o.status(), "carrinho vazio");
        assert_eq!(o.total_cents(), None);
    }

    #[test]
    fn cannot_ship_unpaid_order() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let o = o.ship("BR123");
        assert_eq!(o.status(), "carrinho");
    }

    #[test]
    fn cannot_add_items_after_paying() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let mut o = o.pay();
        assert!(!o.add_item(item("caneta", 350)));
        assert_eq!(o.total_cents(), Some(5000));
    }

    #[test]
    fn cancel_before_shipping_only() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let cancelled = o.clone().pay().cancel("desisti");
        assert_eq!(cancelled.status(), "cancelado");

        let shipped = o.pay().ship("BR1").cancel("tarde demais");
        assert_eq!(shipped.status(), "enviado");
    }
}
