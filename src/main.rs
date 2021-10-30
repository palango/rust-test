mod contracts;
mod monitor;

use contracts::get_contract_data;

use clap::Parser;
use secp256k1::SecretKey;
use tokio::time;

use std::{path::Path, str::FromStr, time::Duration};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, BlockNumber, FilterBuilder, U256},
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(name = "MegaSync")]
struct Config {
    /// Some test
    #[clap(
        short,
        long,
        // default_value = "http://geth.goerli.ethnodes.brainbot.com:8545"
        default_value = "http://127.0.0.1:8545"
    )]
    source_rpc: String,

    #[clap(
        short,
        long,
        default_value = "/Users/paul/Projects/brainbot/raisync/contracts/build/deployments/420"
    )]
    contracts_dir: String,
}

fn get_token_address() -> Address {
    Address::from_str("0x5FbDB2315678afecb367f032d93F642f64180aa3").unwrap()
}

fn get_request_address() -> Address {
    Address::from_str("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0").unwrap()
}

async fn send_events(contract: Contract<Http>) {
    println!("Sending requests");
    let prvk =
        SecretKey::from_str("bbfbee4961061d506ffbb11dfea64eba16355cbf1d9c29613126ba7fec0aed5d")
            .unwrap();

    let source_token_address = get_token_address();
    let target_token_address =
        Address::from_str("0x0000000000000000000000000000000000000002").unwrap();
    let target_address = Address::from_str("0x0000000000000000000000000000000000000003").unwrap();
    let mut amount = 0;

    let mut options = Options::default();
    options.gas = Some(U256::from(1_000_000));

    loop {
        amount += 1;
        let tx = contract
            .signed_call_with_confirmations(
                "request",
                (
                    U256::from(1),
                    source_token_address,
                    target_token_address,
                    target_address,
                    U256::from(amount),
                ),
                options.clone(),
                1,
                &prvk,
            )
            .await;
        println!("Sent tx: {:?}", amount);

        const BETWEEN: Duration = Duration::from_secs(1);
        time::sleep(BETWEEN).await;
    }
}

async fn prepare_token(contract: Contract<Http>) {
    println!("Preparing token");
    let prvk =
        SecretKey::from_str("bbfbee4961061d506ffbb11dfea64eba16355cbf1d9c29613126ba7fec0aed5d")
            .unwrap();
    let sender = Address::from_str("0x66aB6D9362d4F35596279692F0251Db635165871").unwrap();

    let mut options = Options::default();
    options.gas = Some(U256::from(1_000_000));

    let tx = contract
        .signed_call_with_confirmations(
            "mint",
            (sender, U256::from(1_000_000)),
            options.clone(),
            1,
            &prvk,
        )
        .await
        .unwrap();
    println!("got tx: {:?}", tx);

    let tx = contract
        .signed_call_with_confirmations(
            "approve",
            (get_request_address(), U256::from(1_000_000)),
            options.clone(),
            1,
            &prvk,
        )
        .await
        .unwrap();
    println!("got tx: {:?}", tx);
}

#[tokio::main]
async fn main() -> web3::Result {
    env_logger::init();
    let config = Config::parse();
    dbg!(&config);
    let contract_data = get_contract_data(Path::new(&config.contracts_dir));

    println!("Connecting to {}", config.source_rpc);

    let transport = web3::transports::Http::new(&config.source_rpc)?;
    let web3 = web3::Web3::new(transport);

    let block_number = web3.eth().block_number().await?;
    dbg!(&block_number);

    dbg!(contract_data.keys());

    let request_manager = contract_data["RequestManager"].clone();
    let token = &contract_data["MintableToken"];

    let token_contract =
        web3::contract::Contract::from_json(web3.eth(), get_token_address(), &token.get_abi())
            .unwrap();
    let request_contract = web3::contract::Contract::from_json(
        web3.eth(),
        get_request_address(),
        &request_manager.get_abi(),
    )
    .unwrap();

    prepare_token(token_contract).await;

    let h1 = tokio::spawn(async move {
        send_events(request_contract).await;
    });

    let h2 = tokio::spawn(async move {
        filter_events(&request_manager, block_number, web3).await;
    });

    let result = tokio::join!(h1, h2);
    dbg!(result);

    Ok(())
}

async fn filter_events(
    request_manager: &contracts::ContractInfo,
    block_number: web3::types::U64,
    web3: web3::Web3<Http>,
) {
    let contract = web3::ethabi::Contract::load(request_manager.get_abi().as_slice()).unwrap();
    let event = contract.event("RequestCreated").unwrap();
    let filter = FilterBuilder::default()
        .address(vec![get_request_address()])
        .topics(Some(vec![event.signature()]), None, None, None)
        .from_block(BlockNumber::Number(
            request_manager.deployment_info.block_number.into(),
            // block_number - 100,
        ))
        .build();

    loop {
        let a = web3.eth().logs(filter.clone()).await.unwrap();
        println!("Found {} events", a.len());

        const BETWEEN: Duration = Duration::from_secs(1);
        time::sleep(BETWEEN).await;
    }
}
