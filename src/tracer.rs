use async_recursion::async_recursion;
use std::io::Write;
use std::sync::Arc;
use std::{collections::HashMap, fs::File};
use tera::{Context, Tera};
use ton_client::ClientContext;

use crate::{blockchain, jinja};

#[async_recursion]
pub async fn trace_message(
    context: &Arc<ClientContext>,
    abis: &Vec<String>,
    id: &String,
    decode: &bool,
) -> anyhow::Result<blockchain::Message> {
    // Get message
    let message = blockchain::get_message(context, id)
        .await
        .map_err(|e| anyhow::format_err!("Error get message struct {e}"))?;

    // Decode message
    let mut decoded: Option<blockchain::MessageDecodedData> = None;
    if *decode {
        for abi_path in abis {
            decoded = blockchain::decode_message(context, abi_path, &message.boc);
            if !decoded.is_none() {
                break;
            }
        }
    }

    // Parse message child transaction
    let lt_hex = message.dst_transaction.lt.trim_start_matches("0x");
    let lt_i64 = i64::from_str_radix(lt_hex, 16).ok();

    let transaction = blockchain::Transaction {
        id: message.dst_transaction.id,
        aborted: message.dst_transaction.aborted,
        lt: lt_i64,
        exit_code: message.dst_transaction.compute.exit_code,
        vm_steps: message.dst_transaction.compute.vm_steps,
        messages: {
            let mut messages: Vec<blockchain::Message> = Vec::new();
            for msg_id in message.dst_transaction.out_msgs {
                let message = trace_message(context, abis, &msg_id, decode).await?;
                messages.push(message);
            }
            messages
        },
    };

    Ok(blockchain::Message {
        id: message.id,
        src: message.src,
        dst: message.dst,
        boc: message.boc,
        decoded,
        transaction: Some(transaction),
    })
}

pub fn render_trace_template(
    message: &blockchain::Message,
    kwargs: Option<HashMap<&str, String>>,
) -> anyhow::Result<()> {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("index.j2", include_str!("../templates/index.j2")),
        ("macro.j2", include_str!("../templates/macro.j2")),
        ("dist.css", include_str!("../templates/static/dist.css")),
        ("script.js", include_str!("../templates/static/script.js")),
    ])?;
    tera.register_filter("shorten_string", jinja::shorten_string);

    let mut context = Context::new();
    context.insert("message", message);
    if !kwargs.is_none() {
        context.insert("explorer_url", &kwargs.unwrap().get("explorer_url"));
    }
    let html = tera.render("index.j2", &context)?;

    let filename = format!("{}.html", message.id);
    let mut file = File::create(filename)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}
