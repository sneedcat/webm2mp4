pub async fn convert(file_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = format!("{}.mp4", file_name);
    tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(&file_name)
        .arg("-vf")
        .arg("pad=ceil(iw/2)*2:ceil(ih/2)*2")
        .arg(&output)
        .spawn()?
        .wait().await?;
    tokio::fs::remove_file(&file_name).await?;
    Ok(output)
}
