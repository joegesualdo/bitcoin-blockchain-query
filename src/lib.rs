use std::collections::HashMap;

use bitcoind_request::{
    client::Client as BitcoindRequestClient,
    command::{
        get_raw_transaction::{
            GetRawTransactionCommand, GetRawTransactionCommandResponse,
            Transaction as BitcoindTransaction, Vin,
        },
        CallableCommand,
    },
};
use electrs_query::{
    self, get_balance_for_address, get_historical_transactions_for_address, get_utxos_for_address,
    Client as ElectrsClient,
};

pub type VinIndex = u64;
pub type VoutIndex = u64;

pub type SpentFromTransaction = BitcoindTransaction;
pub type SpentInTransaction = BitcoindTransaction;

#[derive(Debug, Clone)]
pub enum TransactionType {
    Sent(VinIndex, SpentFromTransaction, SpentInTransaction),
    Recieved(VoutIndex, BitcoindTransaction),
}
fn get_transaction(
    txid: String,
    bitcoind_request_client: &BitcoindRequestClient,
) -> BitcoindTransaction {
    let transaction_response = GetRawTransactionCommand::new(txid.to_string())
        .verbose(true)
        .call(&bitcoind_request_client)
        .unwrap();
    let transaction_result = match transaction_response {
        GetRawTransactionCommandResponse::Transaction(transaction) => Ok(transaction),
        _ => Err("shouldn't reach"),
    };
    transaction_result.unwrap()
}

fn get_raw_transactions_for_address(
    address: &str,
    electrs_client: &ElectrsClient,
    bitcoind_request_client: &BitcoindRequestClient,
) -> Vec<BitcoindTransaction> {
    let history = get_historical_transactions_for_address(&address, &electrs_client);
    let transactions: Vec<BitcoindTransaction> = history
        .iter()
        .map(|historical_transaction| {
            let txid = &historical_transaction.tx_hash;
            let transaction_response = GetRawTransactionCommand::new(txid.to_string())
                .verbose(true)
                .call(&bitcoind_request_client)
                .unwrap();
            let transaction_result = match transaction_response {
                GetRawTransactionCommandResponse::Transaction(transaction) => Ok(transaction),
                _ => Err("shouldn't reach"),
            };
            transaction_result.unwrap()
        })
        .collect();
    transactions
}

pub fn get_all_transactions_for_address(
    address: &str,
    electrs_client: &ElectrsClient,
    bitcoind_request_client: &BitcoindRequestClient,
) -> Vec<(BitcoindTransaction, Vec<TransactionType>)> {
    let mut all_transactions = vec![];
    let mut transaction_hash: HashMap<String, Vec<TransactionType>> = HashMap::new();
    let balance = get_balance_for_address(&address, &electrs_client);
    let utxos = get_utxos_for_address(&address, &electrs_client);
    let history = get_historical_transactions_for_address(&address, &electrs_client);
    let transactions =
        get_raw_transactions_for_address(&address, &electrs_client, &bitcoind_request_client);
    for tx in &transactions {
        let mut grouped_transactions = vec![];
        for vout in tx.vout.clone() {
            let vout_address = if vout.script_pub_key.address.is_some() {
                vout.script_pub_key.address
            } else {
                vout.address
            };
            match vout_address {
                Some(addr) => {
                    if addr == address {
                        grouped_transactions.push(TransactionType::Recieved(vout.n, tx.clone()));
                    }
                }
                None => {}
            }
        }
        for vin in tx.vin.clone() {
            match vin {
                Vin::Coinbase(vin) => {
                    todo!()
                }
                Vin::NonCoinbase(vin) => {
                    let transaction_for_vin = get_transaction(vin.txid, &bitcoind_request_client);
                    let vout_for_vin = &transaction_for_vin.vout[vin.vout as usize];
                    let vout_address = if vout_for_vin.script_pub_key.address.is_some() {
                        &vout_for_vin.script_pub_key.address
                    } else {
                        &vout_for_vin.address
                    }
                    .clone();
                    match vout_address {
                        Some(addr) => {
                            if addr == address {
                                grouped_transactions.push(TransactionType::Sent(
                                    vout_for_vin.n,
                                    transaction_for_vin.clone(),
                                    tx.clone(),
                                ));
                            }
                        }
                        None => {}
                    }
                }
            }
        }
        all_transactions.push((tx.clone(), grouped_transactions));
    }

    all_transactions
}
