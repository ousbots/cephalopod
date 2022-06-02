use crate::parse::{Tx, Type};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub locked: bool,
    pub history: HashMap<u32, f32>,
}

/// Processes the transactions and updates the ledger of client accounts. Returns the ledger
/// as a hashmap of account id -> account struct.
/// Notes:
///   I couldn't find anything that would legally compel a processor to accept deposits or
///   withdrawals from an account that it considers fraudulent, but the Uniform Commercial Code
///   (UCC) does legally compel a processor to return funds in a chargeback (Section 4-214).
///   
///   Therefore I'm making the assumption that it is better to reject withdrawals from frozen
///   accounts and move that settlement to human interactions, but allow deposits
pub async fn process(mut txs: mpsc::Receiver<Tx>) -> HashMap<u16, Account> {
    // Ideally this is a remote key-value store that can evict old history records after the
    // chargeback time limits have passed. The hashmap is used as a representation of that in
    // this toy program.
    let mut accts = HashMap::<u16, Account>::new();

    while let Some(tx) = txs.recv().await {
        match tx.typ {
            // Searching online it seems common for banks to allow deposits to frozen accounts.
            Type::Deposit => {
                let entry = accts.entry(tx.client).or_insert(Account {
                    client: tx.client,
                    available: 0.,
                    held: 0.,
                    locked: false,
                    history: HashMap::new(),
                });
                entry.available += f64::from(tx.amount);
                entry.history.insert(tx.id, tx.amount);
            }

            Type::Withdrawal => {
                let entry = accts.entry(tx.client).or_insert(Account {
                    client: tx.client,
                    available: 0.,
                    held: 0.,
                    locked: false,
                    history: HashMap::new(),
                });
                if !entry.locked {
                    entry.available -= f64::from(tx.amount);
                    entry.history.insert(tx.id, tx.amount);
                }
            }

            Type::Dispute => {
                if let Some(entry) = accts.get_mut(&tx.client) {
                    if let Some(disputed) = entry.history.get(&tx.id) {
                        entry.available -= f64::from(*disputed);
                        entry.held += f64::from(*disputed);
                    }
                }
            }

            Type::Resolve => {
                if let Some(entry) = accts.get_mut(&tx.client) {
                    if let Some(disputed) = entry.history.get(&tx.id) {
                        entry.held -= f64::from(*disputed);
                        entry.available += f64::from(*disputed);
                    }
                }
            }

            // From reading UCC section 4-214 (https://www.law.cornell.edu/ucc/4/4-214), it seems
            // that a chargeback is legally unavoidable, so no check on account locked status or
            // if sufficient funds are available.
            Type::Chargeback => {
                if let Some(entry) = accts.get_mut(&tx.client) {
                    if let Some(disputed) = entry.history.get(&tx.id) {
                        entry.held -= f64::from(*disputed);
                        entry.locked = true;
                    }
                }
            }
        }
    }

    accts
}
