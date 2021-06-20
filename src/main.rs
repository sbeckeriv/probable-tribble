mod account;
mod transaction;
use account::Account;
use clap::{App, Arg};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;
use transaction::Transaction;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum TransactionType {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct CsvRecord {
    #[serde(alias = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

fn argument_parse() -> clap::ArgMatches<'static> {
    App::new("resolution")
        .version("0.1.0")
        .author("Becker")
        .arg(
            Arg::with_name("INPUT")
                .help("Input csv file")
                .required(true)
                .index(1),
        )
        .get_matches()
}

fn main() {
    let arguments = argument_parse();
    if let Some(path) = arguments.value_of("INPUT") {
        if std::path::Path::new(path).is_file() {
            let mut accounts = HashMap::new();
            let file = std::fs::File::open(path).expect("Could not access csv file");
            let mut reader = csv::ReaderBuilder::new()
                .trim(csv::Trim::All)
                .from_reader(file);
            for result in reader.deserialize() {
                let record: CsvRecord = result.expect("Could not parse csv line");
                let account = accounts
                    .entry(record.client)
                    .or_insert_with(|| Account::new(record.client));

                if record.amount.unwrap_or_default().scale() > 4 {
                    eprintln!("Amount scale is larger then allowed input {:?}", record);
                    continue;
                }
                let transaction = Transaction {
                    id: record.tx,
                    amount: record.amount.unwrap_or_default(),
                    state: record
                        .transaction_type
                        .try_into()
                        .expect("Should always match"),
                    note: "".into(),
                };
                account.add_transaction(transaction);
            }

            println!("client,available,held,total,locked");
            for account in accounts.values() {
                println!("{}", account);
            }
        } else {
            eprintln!("No file found at {}", path);
        }
    }
}
