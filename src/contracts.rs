use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use web3::types::Address;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeploymentInfo {
    pub address: Address,
    #[serde(rename = "blockHeight")]
    pub block_number: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractInfo {
    #[serde(rename = "contractName")]
    pub name: String,
    pub abi: serde_json::Value,
    #[serde(rename = "deployment")]
    pub deployment_info: DeploymentInfo,
}

impl ContractInfo {
    pub fn get_abi(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self.abi).unwrap();
        json.as_bytes().to_owned()
    }
}

pub fn get_contract_data(path: &Path) -> HashMap<String, ContractInfo> {
    let mut result = HashMap::new();
    for item in path.read_dir().unwrap() {
        let item_path = item.unwrap().path();
        dbg!(&item_path);

        // Open the file in read-only mode with buffer.
        let file = File::open(&item_path).unwrap();
        let reader = BufReader::new(file);

        let u: ContractInfo = serde_json::from_reader(reader).unwrap();
        result.insert(u.name.clone(), u);
    }
    result

    // path.read_dir()
    //     .unwrap()
    //     .into_iter()
    //     .map(|x| x.unwrap().path())
    //     .map(|path| {
    //         let file = File::open(&path).unwrap();
    //         let reader = BufReader::new(file);

    //         let u: ContractInfo = serde_json::from_reader(reader).unwrap();

    //         (u.name.clone(), u)
    //     })
    //     .collect()
}
