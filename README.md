# Bitcoin Blockchain Query 
> Request information from the Bitcoin Blockchain

This library provides helpful functions to get information from the Bitcoin Blockhain

---

**⚠️ This is experimental. Please use at your own risk.⚠️**

---

# Requirements
This library relies on querying both a bitcoin full node and an electrs server so you must have both installed and setup to accept incomming rpc commands.

## Install
> Add package to Cargo.toml file
```rust
[dependencies]
bitcoin-blockchain-query = "0.1.3"
```

## Usage:
```rust
use bitcoin_blockchain_query::get_all_transactions_for_address;
use bitcoind_request::{self, client::Client as BitcoindRequestClient};
use electrs_query::{self, Client as ElectrsClient};

fn main() {
    let password = env::var("BITCOIND_PASSWORD").expect("BITCOIND_PASSWORD env variable not set");
    let username = env::var("BITCOIND_USERNAME").expect("BITCOIND_USERNAME env variable not set");
    let bitcoind_url = env::var("BITCOIND_URL").expect("BITCOIND_URL env variable not set");
    let electrs_url = env::var("ELECTRS_URL").expect("ELECTRS_URL env variable not set");

    // Connect to bitcoin full node
    let bitcoind_request_client =
        BitcoindRequestClient::new(&bitcoind_url, &username, &password).unwrap();
    // Connect to electrs server
    let electrs_client = ElectrsClient::new(&electrs_url);

    let transactions = get_all_transactions_for_address(
        "mtveoXKcb1EjpspMmhPAJ6RkGeewbzWYDd",
        &electrs_client,
        &bitcoind_request_client,
    );
    println!("{:#?}", transactions)
}
```

## API
Find a list of all the functions available in the [documentation](https://docs.rs/bitcoin-blockchain-query/latest/bitcoin_blockchain_query/)

## Related
- [electrs-query](https://github.com/joegesualdo/electrs-query) - Query and Electrum server for information
- [electrs-request](https://github.com/joegesualdo/electrs-request) - Type-safe wrapper around electrs RPC commands
- [bitcoin-node-query](https://github.com/joegesualdo/bitcoin-node-query) - Query Bitcoin Node for information
- [bitcoind-request](https://github.com/joegesualdo/bitcoind-request) - Type-safe wrapper around bitcoind RPC commands
- [bitcoin-terminal-dashboard](https://github.com/joegesualdo/bitcoin-terminal-dashboard) - Bitcoin Dashboard in the terminal

## License
MIT © [Joe Gesualdo]()
