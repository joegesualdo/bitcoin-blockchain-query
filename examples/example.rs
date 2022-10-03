use bitcoin_blockchain_query::get_all_transactions_for_address;
use bitcoind_request::{self, client::Client as BitcoindRequestClient};
use electrs_query::{self, Client as ElectrsClient};
use std::env;
fn main() {
    let password = env::var("BITCOIND_PASSWORD").expect("BITCOIND_PASSWORD env variable not set");
    let username = env::var("BITCOIND_USERNAME").expect("BITCOIND_USERNAME env variable not set");
    let bitcoind_url = env::var("BITCOIND_URL").expect("BITCOIND_URL env variable not set");
    let electrs_url = env::var("ELECTRS_URL").expect("ELECTRS_URL env variable not set");

    let bitcoind_request_client =
        BitcoindRequestClient::new(&bitcoind_url, &username, &password).unwrap();
    let electrs_client = ElectrsClient::new(&electrs_url);

    let transactions = get_all_transactions_for_address(
        "mtveoXKcb1EjpspMmhPAJ6RkGeewbzWYDd",
        &electrs_client,
        &bitcoind_request_client,
    );
    println!("{:#?}", transactions)
}
