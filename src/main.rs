use std::env;
use convert::convert_from_webm;
use download::download_file;
use futures::StreamExt;
use telegram_bot::*;

mod download;
mod convert;

async fn process_file(token: &str, file_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let input_webm = download_file(token, file_id).await?;
    let output_mp4 = convert_from_webm(input_webm).await?;
    Ok(output_mp4)
}

async fn process_message(token: &str, file_name: Option<String>, file_id: &str, message: &Message, api: &Api) -> Result<(), Box<dyn std::error::Error>>  {
    let base_name = match file_name {
        Some(s) => {
            let file_name = s;
            std::path::Path::new(&file_name).file_stem().unwrap().to_str().unwrap().to_owned()
        }
        None => "output".to_owned()
    };
    match process_file(&token, &file_id).await {
        Ok(buf) => {
            println!("Done!");
            api.send(message.video_reply(InputFileUpload::with_data(buf, format!("{}.mp4", base_name)))).await?;
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
                    process_message(&cloned_token.clone(), data.file_name.clone(), &data.file_id, &message, &api).await?;
                }
            }
        }
    }
    Ok(())
}
