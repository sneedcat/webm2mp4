use std::{collections::HashSet, error::Error};

use rand::Rng;
use reqwest::Response;
use teloxide::{net::Download, prelude::*, types::{InputFile, MediaKind, MessageEntityKind, MessageKind}};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::convert::convert;
mod convert;

async fn handle_request(file_id: &str, message: &UpdateWithCx<AutoSend<Bot>, Message>) -> Result<(), Box<dyn Error + Send + Sync>>{
    let file = message.requester.get_file(file_id).await?;
    let number: usize = rand::thread_rng().gen();
    let file_name = format!("{}", number);
    let mut resulting_file = File::create(&file_name).await?;
    message.requester.download_file(&file.file_path, &mut resulting_file).await?;
    let resulting_file = convert(&file_name).await?;
    message.reply_video(InputFile::File(resulting_file.clone().into())).await?;
    tokio::fs::remove_file(&resulting_file).await?;
    Ok(())
}

async fn handle_resp(resp: Response, message: &UpdateWithCx<AutoSend<Bot>, Message>) -> Result<(), Box<dyn Error + Send + Sync>>{
    let bytes = &resp.bytes().await?[..];
    let number: usize = rand::thread_rng().gen();
    let file_name = format!("{}", number);
    let mut file = tokio::fs::File::create(&file_name).await?;
    file.write_all(&bytes).await?;
    let resulting_file = convert(&file_name).await?;
    message.reply_video(InputFile::File(resulting_file.clone().into())).await?;
    tokio::fs::remove_file(&resulting_file).await?;
    Ok(())
}

async fn handle_links(links: HashSet<String>, message: &UpdateWithCx<AutoSend<Bot>, Message>) {
    for link in links {
        let resp = match reqwest::get(&link).await {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        if let Some(header) = &resp.headers().get("content-type") {
            if header.to_str().unwrap() == "video/webm" {
                println!("{:?}", handle_resp(resp, message).await);
            } else {
                continue;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting dices_bot...");

    let bot = Bot::from_env().auto_send();

    teloxide::repl(bot, |message| async move {
        match &message.update.kind {
            MessageKind::Common(ref common) => {
                match &common.media_kind {
                    MediaKind::Document(ref doc) => {
                        match &doc.document.mime_type {
                            Some(mime) => {
                                if mime.subtype() == "x-matroska" || mime.subtype() == "webm" {
                                    let res = handle_request(&doc.document.file_id, &message).await;
                                    if let Err(_) = res {
                                        message.reply_to("Something went wrong").await?;
                                    }
                                }
                            }
                            _ => (),
                        }
                    },
                    MediaKind::Text(ref text) => {
                        let mut set = HashSet::new();
                        for entity in &text.entities {
                            if entity.kind == MessageEntityKind::Url {
                                let url = text.text[entity.offset..entity.offset+entity.length].to_owned();
                                set.insert(url);
                            }
                        }
                        handle_links(set, &message).await;
                    }
                    _ => (),
                }
            },
            _ => (),
        }
        respond(())
    })
    .await;
}
