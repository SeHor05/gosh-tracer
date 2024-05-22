use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ton_client::abi::{DecodedMessageBody, ParamsOfDecodeAccountData, ParamsOfDecodeMessage};
use ton_client::net::{query_collection, NetworkConfig, ParamsOfQueryCollection};
use ton_client::{ClientConfig, ClientContext};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: String,
    pub src: String,
    pub dst: String,
    pub boc: String,
    pub decoded: Option<MessageDecodedData>,
    pub transaction: Option<Transaction>,
}

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: String,
    pub aborted: bool,
    pub lt: Option<i64>,
    pub exit_code: Option<i64>,
    pub vm_steps: Option<i64>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
pub struct MessageDecodedData {
    abi_path: String,
    data: DecodedMessageBody,
}

#[derive(Debug, Serialize)]
pub struct Account {
    pub id: String,
    pub boc: String,
    pub code: String,
    pub data: String,
    pub decoded: Option<AccountDecodedData>,
}

#[derive(Debug, Serialize)]
pub struct AccountDecodedData {
    abi_path: String,
    data: Value,
}

#[derive(Deserialize)]
pub struct GraphQLMessage {
    pub id: String,
    pub src: String,
    pub dst: String,
    pub boc: String,
    pub dst_transaction: GraphQLTransaction,
}

#[derive(Deserialize)]
pub struct GraphQLTransaction {
    pub id: String,
    pub aborted: bool,
    pub out_msgs: Vec<String>,
    pub lt: String,
    pub compute: GraphQLTransactionCompute,
}

#[derive(Deserialize)]
pub struct GraphQLTransactionCompute {
    pub exit_code: Option<i64>,
    pub vm_steps: Option<i64>,
}

#[derive(Deserialize)]
pub struct GraphQLAccount {
    pub id: String,
    pub boc: Option<String>,
    pub code: String,
    pub data: String,
}

pub fn get_client_context(endpoints: Vec<String>) -> anyhow::Result<Arc<ClientContext>> {
    let config = ClientConfig {
        network: NetworkConfig {
            endpoints: Some(endpoints),
            ..Default::default()
        },
        ..Default::default()
    };
    let context = ClientContext::new(config)?;
    Ok(Arc::new(context))
}

pub fn get_abi_files(abi_path: &String) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();

    for file in WalkDir::new(abi_path)
        .into_iter()
        .filter_map(|file| file.ok())
    {
        let path = file.path().as_os_str().to_str().unwrap_or("");
        if file.metadata().unwrap().is_file() {
            if path.contains(".abi") || path.contains(".abi.json") {
                paths.push(path.to_string());
            }
        }
    }
    paths
}

pub async fn get_account(
    context: &Arc<ClientContext>,
    id: &String,
) -> anyhow::Result<GraphQLAccount> {
    let response = query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: "accounts".to_string(),
            filter: Some(json!({"id": {"eq": id}})),
            result: "id boc code data".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Error request account");

    let value = response.result.get(0).expect("Account not found");
    let account = serde_json::from_value(value.clone())
        .map_err(|e| anyhow::format_err!("Error deserializing account {e}"))?;
    Ok(account)
}

pub async fn get_message(
    context: &Arc<ClientContext>,
    id: &String,
) -> anyhow::Result<GraphQLMessage> {
    let response = query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: "messages".to_string(),
            filter: Some(json!({"id": {"eq": id}})),
            result:
                "id src dst boc dst_transaction {id aborted out_msgs lt compute {exit_code vm_steps}}"
                    .to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Error request message");

    let value = response.result.get(0).expect("Message not found");
    let message = serde_json::from_value(value.clone())
        .map_err(|e| anyhow::format_err!("Error deserializing message {e}"))?;
    Ok(message)
}

pub fn decode_message(
    context: &Arc<ClientContext>,
    abi_path: &String,
    boc: &String,
) -> Option<MessageDecodedData> {
    let mut json = String::new();
    match File::open(abi_path.clone()) {
        Ok(mut file) => match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    let mut decoded: Option<MessageDecodedData> = None;
    if let Ok(data) = ton_client::abi::decode_message(
        context.clone(),
        ParamsOfDecodeMessage {
            abi: ton_client::abi::Abi::Json(json),
            message: boc.clone(),
            allow_partial: true,
            ..Default::default()
        },
    ) {
        decoded = Some(MessageDecodedData {
            abi_path: abi_path.clone(),
            data,
        });
    }

    decoded
}

pub fn decode_account_data(
    context: &Arc<ClientContext>,
    abi_path: &String,
    data: &String,
) -> Option<AccountDecodedData> {
    let mut json = String::new();
    match File::open(abi_path.clone()) {
        Ok(mut file) => match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    let mut decoded: Option<AccountDecodedData> = None;
    if let Ok(data) = ton_client::abi::decode_account_data(
        context.clone(),
        ParamsOfDecodeAccountData {
            abi: ton_client::abi::Abi::Json(json),
            data: data.clone(),
            allow_partial: true,
            ..Default::default()
        },
    ) {
        decoded = Some(AccountDecodedData {
            abi_path: abi_path.clone(),
            data: data.data,
        });
    }

    decoded
}
