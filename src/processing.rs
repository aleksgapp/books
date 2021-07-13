use super::engine::{Account, ClientAssets, Transactions, Status};
use super::transactions::*;
use super::money::Money;

pub trait PaymentsProcessing {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions);
}

impl PaymentsProcessing for Deposit {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions) {
        let mut success = true;
        accounts.entry(self.0.meta.client_id).and_modify(|account| {
            if account.locked {
                log::warn!("deposit attempt to a locked accout <tx_id: {}>", self.0.meta.tx_id);
                success = false;
                return;
            }
            account.available += self.0.amount;
        }).or_insert_with(|| Account { 
            available: self.0.amount,
            held: Money::zero(), 
            locked: false 
        });

        if success {
            txs.insert(self.0.meta.tx_id, (Transaction::Deposit(*self), Status::Normal));
        }
    }
}

impl PaymentsProcessing for Withdrawal {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions) {
        if self.0.amount < Money::zero() {
            log::warn!("negative withdrowal amount <tx_id: {}>", self.0.meta.tx_id);
            return;
        }

        accounts.entry(self.0.meta.client_id).and_modify(|account| {
            if account.locked {
                log::warn!("withdrawal attempt from a locked accout <tx_id: {}>", self.0.meta.tx_id);
                return;
            }

            if account.available < self.0.amount {
                log::warn!("inefficient funds for withdrowal <tx_id: {}>", self.0.meta.tx_id);
                return;
            }

            account.available -= self.0.amount;

            txs.insert(self.0.meta.tx_id, (Transaction::Withdrawal(*self), Status::Normal));
        });
    }
}

impl PaymentsProcessing for Dispute {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions) {
        if let Some((tx, status)) = txs.get_mut(&self.0.tx_id) {
            if !status.is_normal() {
                return;
            }
            if let Some(amount) = tx.get_amount() {
                accounts.entry(self.0.client_id).and_modify(|account| {
                    if account.locked {
                        log::warn!("dispute attempt for a locked accout <tx_id: {}>", self.0.tx_id);
                        return;
                    }
    
                    if account.available < amount {
                        // TODO(gpl): not enough funds available for putting on hold
                        // not sure what bahaviour is expected in this case
                        log::error!("inefficient funds for a dispute <tx_id: {}>", self.0.tx_id);
                        return;
                    }

                    account.available -= amount;
                    account.held += amount;

                    *status = Status::Disputed;
                });
            }
        }
    }
}

impl PaymentsProcessing for Resolve {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions) {
        if let Some((tx, status)) = txs.get_mut(&self.0.tx_id) {
            if !status.is_disputed() {
                return;
            }
            if let Some(amount) = tx.get_amount() {
                accounts.entry(self.0.client_id).and_modify(|account| {
                    if account.locked {
                        log::warn!("resolve attempt for a locked accout <tx_id: {}>", self.0.tx_id);
                        return;
                    }
                    if account.held < amount {
                        // TODO(gpl): shouldn't be possible to get here, since in order to dispute,
                        // we make sure there is enough funds.
                        log::error!("inefficient funds to resolve a dispute <tx_id: {}>", self.0.tx_id);
                        return;
                    }
                    account.available += amount;
                    account.held -= amount;

                    *status = Status::Normal;
                });
            }
        }
    }
}

impl PaymentsProcessing for Chargeback {
    fn process(&self, accounts: &mut ClientAssets, txs: &mut Transactions) {
        if let Some((tx, status)) = txs.get_mut(&self.0.tx_id) {
            if !status.is_disputed() {
                return;
            }
            if let Some(amount) = tx.get_amount() {
                accounts.entry(self.0.client_id).and_modify(|account| {
                    if account.locked {
                        log::warn!("chargeback attempt for a locked accout <tx_id: {}>", self.0.tx_id);
                        return;
                    }
                    if account.held < amount {
                        // TODO(gpl): shouldn't be possible to get here, since in order to dispute,
                        // we make sure there is enough funds.
                        log::error!("inefficient funds for the chargeback <tx_id: {}>", self.0.tx_id);
                        return;
                    }
                    account.held -= amount;
                    account.locked = true;

                    *status = Status::Chargedback;
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_deposit() -> Deposit {
        let client_id = 1;
        let meta = Metadata::new(client_id, 2);
        let amount = Money::from_str(&"1/10").unwrap();
        Deposit::new(meta, amount)
    }

    fn assets_and_txs() -> (ClientAssets, Transactions) {
        (HashMap::new(), HashMap::new())
    }

    #[test]
    fn inserts_new_client() {
        let deposit = test_deposit();

        let (mut accounts, mut txs) = assets_and_txs();
        deposit.process(&mut accounts, &mut txs);

        let assets = accounts.get(&deposit.0.meta.client_id).unwrap();
        assert_eq!(assets.available, deposit.0.amount);
    }

    #[test]
    fn updates_existing_client() {
        let deposit = test_deposit();

        let (mut accounts, mut txs) = assets_and_txs();
        accounts.insert(deposit.0.meta.client_id, Account {
            available: deposit.0.amount,
            held: Money::zero(),
            locked: false,
        });
        deposit.process(&mut accounts, &mut txs);

        let assets = accounts.get(&deposit.0.meta.client_id).unwrap();
        assert_eq!(assets.available, deposit.0.amount+deposit.0.amount);
    }

    #[test]
    fn cant_deposit_locked_account() {
        let deposit = test_deposit();

        let (mut accounts, mut txs) = assets_and_txs();
        accounts.insert(deposit.0.meta.client_id, Account {
            available: Money::zero(),
            held: Money::zero(),
            locked: true,
        });
        deposit.process(&mut accounts, &mut txs);

        let assets = accounts.get(&deposit.0.meta.client_id).unwrap();
        assert_eq!(assets.available, Money::zero());
    }
}