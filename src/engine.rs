use super::money::Money;
use super::transactions::*;
use super::processing::PaymentsProcessing;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Account {
    pub available: Money,
    pub held: Money,
    pub locked: bool,
}

impl Account {
    pub fn total(&self) -> Money {
        self.available + self.held
    }
}

pub type ClientAssets = HashMap<ClienID, Account>;

#[derive(Debug)]
pub enum Status {
    Normal,
    Disputed,
    Chargedback,
}
impl Status {
    pub fn is_normal(&self) -> bool {
        match self {
            Status::Normal => true,
            _ => false,
        }
    }
    pub fn is_disputed(&self) -> bool {
        match self {
            Status::Disputed => true,
            _ => false,
        }
    }
}

pub type Transactions = HashMap<TxID, (Transaction, Status)>;

#[derive(Debug)]
pub struct ToyPaymentsEngine {
    assets: ClientAssets,
    txs: Transactions,
}

impl ToyPaymentsEngine {
    pub fn new() -> Self {
        ToyPaymentsEngine {
            assets: HashMap::new(),
            txs: HashMap::new()
        }
    }
    fn do_process(&mut self, tx: impl PaymentsProcessing) {
        tx.process(&mut self.assets, &mut self.txs);
    }
    pub fn process(&mut self, tx: Transaction) {
        match tx {
            Transaction::Deposit(deposit) => self.do_process(deposit),
            Transaction::Withdrawal(withdrawal) => self.do_process(withdrawal),
            Transaction::Dispute(dispute) => self.do_process(dispute),
            Transaction::Resolve(resolve) => self.do_process(resolve),
            Transaction::Chargeback(chargeback) => self.do_process(chargeback),
        };
    }
    pub fn assets(&self) -> &ClientAssets {
        &self.assets
    }
}