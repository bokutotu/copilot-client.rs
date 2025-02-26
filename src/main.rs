use std::error::Error;

mod client;

use client::{CopilotClient, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // from_env() で内部から GitHub トークンを取得
    let editor_version = "Neovim/0.9.0".to_string();
    let client = CopilotClient::from_env(editor_version)?;

    // エージェント情報を取得
    let agents = client.get_agents().await?;
    println!("Agents: {agents:?}");

    // モデル情報を取得
    let models = client.get_models().await?;
    println!("Models: {models:?}");

    // チャットリクエスト例
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "あなたは優秀なアシスタントです。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "RustでHTTPリクエストを送る方法を教えてください。".to_string(),
        },
    ];
    let chat_response = client
        .chat_completion(messages, "gemini-2.0-flash-001".to_string())
        .await?;
    println!("Chat Response: {chat_response:?}");

    // 埋め込み生成リクエスト例
    let inputs = vec!["Rust programming language".to_string()];
    let embeddings = client.get_embeddings(inputs).await?;
    println!("Embeddings: {embeddings:?}");

    Ok(())
}
