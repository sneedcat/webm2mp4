use std::time::UNIX_EPOCH;
use std::process::Stdio;

pub async fn convert_from_webm(buf: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let time = std::time::SystemTime::now();
    let nanos = time
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let input_filename = format!("{}.webm", nanos);
    let output_filename = format!("{}.mp4", nanos);
    tokio::fs::write(&input_filename, buf).await?;
    let mut command = tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_filename)
        .arg("-vf")
        .arg("pad=ceil(iw/2)*2:ceil(ih/2)*2")
        .arg(&output_filename)
        .spawn()?;
    let status = command.wait().await?;
    let buf = tokio::fs::read(&output_filename).await?;
    tokio::fs::remove_file(&input_filename).await?;
    tokio::fs::remove_file(&output_filename).await?;
    Ok(buf)
}