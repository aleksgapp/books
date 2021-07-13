use super::money::{Money, MoneyVisitor};
use serde::de::Deserializer;
use serde::Deserialize;
use serde::de;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all="lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub type ClienID = u16;
pub type TxID = u32;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Metadata {
    pub client_id: ClienID,
    pub tx_id: TxID,
}

impl Metadata {
    pub fn new(client_id: ClienID, tx_id: TxID) -> Self {
        Metadata {
            client_id: client_id,
            tx_id: tx_id
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Transfer {
    pub meta: Metadata,
    pub amount: Money,
}

impl Transfer {
    pub fn new(meta: Metadata, amount: Money) -> Self {
        Transfer {
            meta: meta,
            amount: amount,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Deposit(pub Transfer);
impl Deposit {
    pub fn new(meta: Metadata, amount: Money) -> Self {
        Deposit(Transfer::new(meta, amount))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Withdrawal(pub Transfer);
impl Withdrawal {
    pub fn new(meta: Metadata, amount: Money) -> Self {
        Withdrawal(Transfer::new(meta, amount))
    }
}

#[derive(PartialEq, Debug)]
pub struct Dispute(pub Metadata);
impl Dispute {
    pub fn new(meta: Metadata) -> Self {
        Dispute(meta)
    }
}

#[derive(PartialEq, Debug)]
pub struct Resolve(pub Metadata);
impl Resolve {
    pub fn new(meta: Metadata) -> Self {
        Resolve(meta)
    }
}

#[derive(PartialEq, Debug)]
pub struct Chargeback(pub Metadata);
impl Chargeback {
    pub fn new(meta: Metadata) -> Self {
        Chargeback(meta)
    }
}

#[derive(PartialEq, Debug)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

impl Transaction {
    pub fn get_amount(&self) -> Option<Money> {
        match self {
            Transaction::Deposit(deposit) => Some(deposit.0.amount),
            Transaction::Withdrawal(withdrawal) => Some(withdrawal.0.amount),
            _ => None
        }
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
    where 
        D: Deserializer<'de> 
    {
        #[derive(Deserialize)]
        #[serde(rename_all="lowercase")]
        enum Type {
            Deposit,
            Withdrawal,
            Dispute,
            Resolve,
            Chargeback,
        }

        #[derive(Deserialize)]
        struct Event {
            #[serde(alias="type")]
            tx_type: Type,

            #[serde(alias="client")]
            client_id: u16,

            #[serde(alias="tx")]
            tx_id: u32,

            #[serde(deserialize_with = "str_to_money")]
            amount: Option<Money>,
        }

        let event = Event::deserialize(deserializer)?;
        let meta = Metadata::new(event.client_id, event.tx_id);
        match event.tx_type {
            Type::Deposit => {
                event.amount
                .ok_or_else(|| { 
                    log::debug!("deposit tx without an amount <tx_id: {}>", event.tx_id);
                    de::Error::custom("invalid deposit tx") 
                })
                .map(|val| Deposit::new(meta, val))
                .map(Transaction::Deposit)
            },
            Type::Withdrawal => {
                event.amount
                .ok_or_else(|| { 
                    log::debug!("withdrawal tx without an amount <tx_id: {}>", event.tx_id);
                    de::Error::custom("invalid withdrawal tx") 
                })
                .map(|val| Withdrawal::new(meta, val))
                .map(Transaction::Withdrawal)
            }
            Type::Dispute => Ok(Dispute::new(meta)).map(Transaction::Dispute),
            Type::Resolve => Ok(Resolve::new(meta)).map(Transaction::Resolve),
            Type::Chargeback => Ok(Chargeback::new(meta)).map(Transaction::Chargeback),
        }
    }
}

fn str_to_money<'de, D>(deserializer: D) -> Result<Option<Money>, D::Error> 
where 
    D: Deserializer<'de> 
{
    deserializer.deserialize_str(MoneyVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_parsing() {
        let csv = "type,client,tx,amount
deposit,1,1,1.10015
withdrawal,1,4,
dispute,1,2,
resolve,1,2,
chargeback,1,2,
deposit,1,2,1
withdrawal,1,4,1.5";

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let txs: Vec<Transaction> = reader.deserialize()
            .filter(|r| r.is_ok()).map(|r| r.unwrap()).collect();

        assert_eq!(
            txs[0], 
            Transaction::Deposit(
                Deposit::new(
                    Metadata::new(1, 1),
                    Money::from_str("11001/10000").unwrap())));
        
        assert_eq!(txs[1], Transaction::Dispute(Dispute::new(Metadata::new(1, 2))));
        assert_eq!(txs[2], Transaction::Resolve(Resolve::new(Metadata::new(1, 2))));
        assert_eq!(txs[3], Transaction::Chargeback(Chargeback::new(Metadata::new(1, 2))));

        assert_eq!(
            txs[4],
            Transaction::Deposit(
                Deposit::new(
                    Metadata::new(1, 2), 
                    Money::from_str("1/1").unwrap())));

        assert_eq!(
            txs[5],
            Transaction::Withdrawal(
                Withdrawal::new(
                    Metadata::new(1, 4), 
                    Money::from_str("3/2").unwrap())));
    }
}