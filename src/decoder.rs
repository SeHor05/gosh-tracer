use std::{fs::File, io::Write, sync::Arc};

use ton_client::ClientContext;

use crate::blockchain;

pub async fn decode_account(
    context: &Arc<ClientContext>,
    abis: &Vec<String>,
    id: &String,
) -> anyhow::Result<blockchain::Account> {
    // Get account
    let account = blockchain::get_account(context, id)
        .await
        .map_err(|e| anyhow::format_err!("Error get account struct {e}"))?;
    if account.boc.is_none() {
        return Err(anyhow::format_err!("Account has empty BOC"));
    }

    // Decode account data
    let mut decoded: Option<blockchain::AccountDecodedData> = None;
    for abi_path in abis {
        decoded = blockchain::decode_account_data(context, abi_path, &account.data);
        if !decoded.is_none() {
            break;
        }
    }

    Ok(blockchain::Account {
        id: account.id,
        boc: account.boc.unwrap_or(String::from("")),
        code: account.code,
        data: account.data,
        decoded,
    })
}

pub async fn decode_message(
    context: &Arc<ClientContext>,
    abis: &Vec<String>,
    id: &String,
) -> anyhow::Result<blockchain::Message> {
    // Get message
    let message = blockchain::get_message(context, id)
        .await
        .map_err(|e| anyhow::format_err!("Error get message struct {e}"))?;

    // Decode message
    let mut decoded: Option<blockchain::MessageDecodedData> = None;
    for abi_path in abis {
        decoded = blockchain::decode_message(context, abi_path, &message.boc);
        if !decoded.is_none() {
            break;
        }
    }

    Ok(blockchain::Message {
        id: message.id,
        src: message.src,
        dst: message.dst,
        boc: message.boc,
        decoded,
        transaction: None,
    })
}

pub fn render_account(account: &blockchain::Account) -> anyhow::Result<()> {
    let filename = format!("{}.json", account.id);
    let mut file = File::create(filename)?;
    let json = serde_json::to_string_pretty(account)
        .map_err(|e| anyhow::format_err!("Error serializing account {e}"))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn render_message(message: &blockchain::Message) -> anyhow::Result<()> {
    let filename = format!("{}.json", message.id);
    let mut file = File::create(filename)?;
    let json = serde_json::to_string_pretty(message)
        .map_err(|e| anyhow::format_err!("Error serializing message {e}"))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
