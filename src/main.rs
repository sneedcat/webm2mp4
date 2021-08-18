use std::env;
use download::download_file;
use futures::StreamExt;
use telegram_bot::*;

mod download;
mod convert;

async fn process_file(token: &str, file_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let input_webm = download_file(token, file_id).await?;
    Ok(input_webm)
}

async fn process_message(token: &str, file_id: &str, message: &Message, api: &Api) -> Result<(), Box<dyn std::error::Error>>  {
    match process_file(&token, &file_id).await {
        Ok(buf) => {
            println!("Done!");
            api.send(message.video_reply(InputFileUpload::with_data(buf, "output.webm"))).await?;
        }
        Err(_) => {
            api.send(message.text_reply(
                "Something went wrong"
            )).await?;
        }
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let cloned_token = token.clone();
    let api = Api::new(token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Document { ref data , ..} = message.kind {
                println!("{:?}", data.mime_type);
                if data.mime_type == Some("video/webm".to_string()) {
                    process_message(&cloned_token.clone(), &data.file_id, &message, &api).await?;
                }
            }
            if let MessageKind::Video { ref data, .. } = message.kind {
                println!("{:?}", data.mime_type);
                if data.mime_type == Some("video/webm".to_string()) {
                    process_message(&cloned_token.clone(), &data.file_id, &message, &api).await?;
                }
            }
        }
    }
    Ok(())
}
