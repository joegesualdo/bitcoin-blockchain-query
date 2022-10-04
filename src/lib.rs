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
pub enum TransactionFlow {
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

#[derive(Debug, Clone)]
pub struct TransactionFlowsWithTransaction(pub (BitcoindTransaction, Vec<TransactionFlow>));
#[derive(Debug, Clone)]
pub struct TransactionFlowsForAddress(pub Vec<TransactionFlowsWithTransaction>);

type Txid = String;
type Blocktime = i64;
type TransactionFlowsForMultipleAddressesOrganizedByTransaction =
    HashMap<(Txid, Blocktime), Vec<TransactionFlow>>;

pub fn organize_transaction_flows_for_mulitple_addresses_by_txid_and_blocktime(
    transaction_flows_for_addresses: Vec<TransactionFlowsForAddress>,
) -> TransactionFlowsForMultipleAddressesOrganizedByTransaction {
    let mut transactions_grouped_by_transaction: TransactionFlowsForMultipleAddressesOrganizedByTransaction  =
        HashMap::new();
    for transaction_flows_for_address in transaction_flows_for_addresses.clone() {
        for transaction_flows_with_transaction in transaction_flows_for_address.0 {
            let (tx, tx_types) = transaction_flows_with_transaction.0;
            let txid = tx.txid;
            let blocktime = tx.time as i64;
            match transactions_grouped_by_transaction.get(&(txid.clone(), blocktime)) {
                Some(transactions) => {
                    let list_to_add = tx_types.clone();
                    let new_list = transactions.iter().chain(&list_to_add).cloned().collect();
                    transactions_grouped_by_transaction.insert((txid, blocktime), new_list);
                }
                None => {
                    transactions_grouped_by_transaction.insert((txid, blocktime), tx_types);
                }
            }
        }
    }
    transactions_grouped_by_transaction
}

pub fn get_transaction_flows_for_address(
    address: &str,
    electrs_client: &ElectrsClient,
    bitcoind_request_client: &BitcoindRequestClient,
) -> TransactionFlowsForAddress {
    let mut transaction_flows_for_address = vec![];
    let transactions =
        get_raw_transactions_for_address(address, electrs_client, bitcoind_request_client);
    for tx in &transactions {
        let mut flows_for_transaction = vec![];
        for vout in tx.vout.clone() {
            let vout_address = if vout.script_pub_key.address.is_some() {
                vout.script_pub_key.address
            } else {
                vout.address
            };
            match vout_address {
                Some(addr) => {
                    if addr == address {
                        flows_for_transaction.push(TransactionFlow::Recieved(vout.n, tx.clone()));
                    }
                }
                None => {}
            }
        }
        for vin in tx.vin.clone() {
            match vin {
                Vin::Coinbase(_vin) => {
                    todo!()
                }
                Vin::NonCoinbase(vin) => {
                    let transaction_for_vin = get_transaction(vin.txid, bitcoind_request_client);
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
                                flows_for_transaction.push(TransactionFlow::Sent(
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
        transaction_flows_for_address.push(TransactionFlowsWithTransaction((
            tx.clone(),
            flows_for_transaction,
        )));
    }

    TransactionFlowsForAddress(transaction_flows_for_address)
}
