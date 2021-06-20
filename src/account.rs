use super::transaction::*;
use core::fmt;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Account {
    id: u16,
    locked: bool,
    available: Decimal,
    held: Decimal,
    ledger: HashMap<u32, Vec<Transaction>>,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            id,
            ..Default::default()
        }
    }

    // Add the transaction to the ledger and move the current state (balance)
    // along its journey
    pub fn add_transaction(&mut self, transaction: Transaction) {
        if self.is_locked() {
            return;
        }

        let mut transaction = transaction;
        match self.get_action(&transaction) {
            TransactionAction::Lock(change) => {
                self.locked = true;
                // on dispute we took the money from available put it back?
                self.available = self.available + change;
                self.held -= change;
            }
            TransactionAction::Resolve(change) => {
                self.held = self.held - change;
                self.available = self.available + change;
            }

            TransactionAction::Dispute(change) => {
                // what if available goes negative?
                self.held = self.held + change;
                // Why am i removing it available again if it was already withdrawn?
                self.available = self.available - change;
            }

            TransactionAction::RemoveAvailable(change) => {
                if change > self.available {
                    transaction.failed_withdraw(self.available);
                } else {
                    self.available = self.available - change;
                }
            }

            TransactionAction::AddAvailable(change) => {
                self.available = self.available + change;
            }

            TransactionAction::Ignore => {
                // If required logging could go here.
            }
        };

        let list = self.ledger.entry(transaction.id).or_insert_with(|| vec![]);
        list.push(transaction);
    }

    fn get_action(&self, transaction: &Transaction) -> TransactionAction {
        match transaction.state {
            TransactionState::Withdraw | TransactionState::Deposit => transaction.action(),
            _ => {
                let list = self.ledger.get(&transaction.id);

                let ignore = Transaction {
                    state: TransactionState::Ignore,
                    id: 0,
                    amount: Decimal::default(),
                    note: "".to_owned(),
                };
                if let Some(list) = list {
                    let mut base = list.first().unwrap_or(&ignore);
                    let mut temp;

                    if base.state != TransactionState::Withdraw {
                        return ignore.action();
                    }

                    for stored_transaction in list.iter().skip(1) {
                        temp = base.progress(stored_transaction);
                        base = &temp;
                    }
                    base.progress(transaction).action()
                } else {
                    ignore.action()
                }
            }
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
    pub fn total(&self) -> Decimal {
        self.held + self.available
    }
}
impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.id,
            self.available,
            self.held,
            self.total(),
            self.is_locked(),
        )
    }
}
