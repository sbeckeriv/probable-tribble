use super::TransactionType;
use rust_decimal::Decimal;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionState {
    Withdraw,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
    Ignore,
    FailedWithdraw,
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::Ignore
    }
}
impl TryInto<TransactionState> for TransactionType {
    type Error = String;
    fn try_into(self) -> Result<TransactionState, Self::Error> {
        match self {
            TransactionType::Withdrawal => Ok(TransactionState::Withdraw),
            TransactionType::Deposit => Ok(TransactionState::Deposit),
            TransactionType::Dispute => Ok(TransactionState::Dispute),
            TransactionType::Resolve => Ok(TransactionState::Resolve),
            TransactionType::Chargeback => Ok(TransactionState::Chargeback),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionAction {
    AddAvailable(Decimal),
    RemoveAvailable(Decimal),
    Dispute(Decimal),
    Resolve(Decimal),
    Lock(Decimal),
    Ignore,
}

#[derive(Debug, Clone, Default)]
pub struct Transaction {
    pub id: u32,
    pub state: TransactionState,
    pub amount: Decimal,
    pub note: String,
}

impl Transaction {
    // Transactions are a state machine. We will use this as a
    // stand in for a real state machine
    pub fn progress(&self, other: &Transaction) -> Transaction {
        match (&self.state, &other.state) {
            // I did not cover disputing a deposit but it can be done in flow.
            (TransactionState::Deposit, _) => Transaction {
                state: TransactionState::Ignore,
                id: self.id,
                amount: self.amount,
                note: "".to_owned(),
            },
            (TransactionState::Withdraw, TransactionState::Dispute) => Transaction {
                state: TransactionState::Dispute,
                id: self.id,
                amount: self.amount,
                note: "".to_owned(),
            },
            (TransactionState::Dispute, TransactionState::Resolve) => Transaction {
                state: TransactionState::Resolve,
                id: self.id,
                amount: self.amount,
                note: "".to_owned(),
            },
            (TransactionState::Dispute, TransactionState::Chargeback) => Transaction {
                state: TransactionState::Chargeback,
                id: self.id,
                amount: self.amount,
                note: "".to_owned(),
            },
            (_, _) => Transaction {
                state: TransactionState::Ignore,
                id: self.id,
                amount: self.amount,
                note: "".to_owned(),
            },
        }
    }
    pub fn action(&self) -> TransactionAction {
        match self.state {
            TransactionState::Deposit => TransactionAction::AddAvailable(self.amount),
            TransactionState::Withdraw => TransactionAction::RemoveAvailable(self.amount),
            TransactionState::Dispute => TransactionAction::Dispute(self.amount),
            TransactionState::Resolve => TransactionAction::Resolve(self.amount),
            TransactionState::Chargeback => TransactionAction::Lock(self.amount),
            TransactionState::Ignore => TransactionAction::Ignore,
            TransactionState::FailedWithdraw => TransactionAction::Ignore,
        }
    }

    pub fn failed_withdraw(&mut self, amount: Decimal) {
        self.state = TransactionState::FailedWithdraw;
        self.note = format!(
            "Tried to withdraw {} but account had {} available",
            self.amount, amount
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_progress() {
        // would be nice to have a full test matrix
        let deposit = Transaction {
            state: TransactionState::Deposit,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };

        let withdraw = Transaction {
            state: TransactionState::Withdraw,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };
        let dispute = Transaction {
            state: TransactionState::Dispute,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };
        let resolve = Transaction {
            state: TransactionState::Resolve,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };

        let chargeback = Transaction {
            state: TransactionState::Chargeback,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };

        let ignore = Transaction {
            state: TransactionState::Ignore,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };

        let failed_withdraw = Transaction {
            state: TransactionState::FailedWithdraw,
            id: 1,
            amount: Decimal::from_str("2.22").expect("works"),
            note: "".to_owned(),
        };
        assert_eq!(deposit.progress(&withdraw).state, TransactionState::Ignore);
        assert_eq!(withdraw.progress(&deposit).state, TransactionState::Ignore);
        assert_eq!(withdraw.progress(&dispute).state, TransactionState::Dispute);
        assert_eq!(dispute.progress(&resolve).state, TransactionState::Resolve);
        assert_eq!(
            dispute.progress(&chargeback).state,
            TransactionState::Chargeback
        );
    }
}
