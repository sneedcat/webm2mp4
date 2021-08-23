use convert::convert_from_webm;
use download::{download_file, download_file_from_link};
use futures::StreamExt;
use regex::Regex;
use std::{
    collections::HashSet,
    env,
    sync::Arc,
};
use telegram_bot::*;

use log::{info, warn};
use log4rs;

mod convert;
mod download;

async fn process_file(token: &str, file_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let input_webm = download_file(token, file_id).await?;
    let output_mp4 = convert_from_webm(input_webm).await?;
    Ok(output_mp4)
}

async fn process_link(link: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let buf = download_file_from_link(link).await?;
    let output_mp4 = convert_from_webm(buf).await?;
    Ok(output_mp4)
}

async fn process_links (
    map: HashSet<(&str, &str)>,
    message: &Message,
    api: &Api,
) -> Result<(), Box<dyn std::error::Error>> {
    for item in map {
        let buf = match process_link(item.0).await {
            Ok(buf) => buf,
            Err(e) => {
                warn!("{}", e);
                continue;
            }
        };
        let base_name = std::path::Path::new(&item.1)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        match api.send(message.video_reply(InputFileUpload::with_data(
            buf,
            format!("{}.mp4", base_name),
        )))
        .await {
            Ok(_) => {
                info!("Sent video to {}", message.chat.id());
            }
            Err(e) => {
                warn!("Failed to send video to {}: {}", message.chat.id(), e);
            }
        }
    }
    Ok(())
}

async fn process_message(
    token: &str,
    file_name: Option<String>,
    file_id: &str,
    message: &Message,
    api: &Api,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_name = match file_name {
        Some(s) => {
            let file_name = s;
            std::path::Path::new(&file_name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        }
        None => "output".to_owned(),
    };
    match process_file(&token, &file_id).await {
        Ok(buf) => {
            api.send(message.video_reply(InputFileUpload::with_data(
                buf,
                format!("{}.mp4", base_name),
            )))
            .await?;
        }
        Err(_) => {
            api.send(message.text_reply("Something went wrong")).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let token = match env::var("TELEGRAM_BOT_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            warn!("NO TELEGRAM_BOT_TOKEN SET");
            "".to_owned()
        }
    };
    let cloned_token = token.clone();
    let api = Api::new(token);

    let regex = Arc::new(Regex::new(r"https://.*?[/]([^/]*?[.]webm)").unwrap());

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        if let Err(err) = &update {
            warn!("{}", err);
        }
        let regex = Arc::clone(&regex);
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Document { ref data, .. } = message.kind {
                if data.mime_type == Some("video/webm".to_string()) {
                    match process_message(
                        &cloned_token.clone(),
                        data.file_name.clone(),
                        &data.file_id,
                        &message,
                        &api,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Done!");
                        }
                        Err(err) => {
                            warn!("{}", err);
                        }
                    }
                }
            } else if let MessageKind::Text { ref data, .. } = message.kind {
                let matches = regex.captures_iter(data);
                let mut set = HashSet::new();
                for m in matches {
                    let url = m.get(0);
                    if url == None {
                        continue;
                    }
                    let url = url.unwrap().as_str();
                    let filename = m.get(1);
                    if filename == None {
                        continue;
                    }
                    let filename = filename.unwrap().as_str();
                    set.insert((url, filename));
                }
                if set.len() > 0 {
                    process_links(set, &message, &api).await?;
                }
            } else {
                continue;
            }
        }
    }
    Ok(())
}
