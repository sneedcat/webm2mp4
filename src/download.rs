use std::str::FromStr;

use hyper::{Client, Uri, body::HttpBody};
use hyper_tls::HttpsConnector;
use serde::Deserialize;

pub async fn download_file(token: &str, file_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = format!("https://api.telegram.org/bot{}/getFile?file_id={}", token, file_id);

    let mut res = client.get(Uri::from_str(&url)?).await?;
    let mut buf = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        buf.extend_from_slice(&chunk);
    }
    let data =  std::str::from_utf8(&buf)?;

    #[derive(Deserialize)]
    struct Result {
        file_path: String
    }
    #[derive(Deserialize)]
    struct Data {
        result: Result
    }
    
    let data: Data = serde_json::from_str(data)?;
    let file_path = data.result.file_path;
    let url = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);
    //let mut res = client.get(Uri::from_str(&url)?).await?;
    let mut res = client.get(Uri::from_str(&url)?).await?;
    let mut buf = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}