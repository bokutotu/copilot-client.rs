use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tokio;

// ここでは Cargo.toml に以下の依存関係を追加してください
// [dependencies]
// reqwest = { version = "0.11", features = ["json"] }
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// dirs = "4.0"

/// Lua の実装と同様の方法で GitHub トークンを取得する
async fn get_github_token() -> Result<String, Box<dyn Error>> {
    // Codespaces 環境の場合、GITHUB_TOKEN が設定されているならそれを使う
    let token_env = env::var("GITHUB_TOKEN").ok();
    let codespaces = env::var("CODESPACES").ok();
    if let (Some(token), Some(_)) = (token_env, codespaces) {
        return Ok(token);
    }

    // Codespaces でない場合、設定ファイルから取得する
    // 設定ファイルのパスを取得（ここでは dirs クレートを使用）
    let mut config_path: PathBuf = if let Some(dir) = dirs::config_dir() {
        dir
    } else {
        return Err("Failed to find config directory".into());
    };

    // github-copilot 配下の設定ディレクトリを指定
    config_path.push("github-copilot");

    // チェックするファイル一覧
    let file_names = ["hosts.json", "apps.json"];
    for file_name in file_names.iter() {
        let mut file_path = config_path.clone();
        file_path.push(file_name);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            // JSON はキーが不定なオブジェクトとして扱う
            let userdata: HashMap<String, Value> = serde_json::from_str(&content)?;
            for (key, value) in userdata.iter() {
                if key.contains("github.com") {
                    if let Some(oauth_token) = value.get("oauth_token").and_then(|v| v.as_str()) {
                        return Ok(oauth_token.to_string());
                    }
                }
            }
        }
    }
    Err("Failed to find GitHub token".into())
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponseChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatResponseChoice>,
}

/// Copilot API を利用するためのクライアント
struct CopilotClient {
    http_client: reqwest::Client,
    token: String,
    editor_version: String,
}

impl CopilotClient {
    /// 新しいクライアントを生成する
    async fn new() -> Result<Self, Box<dyn Error>> {
        // 環境変数や設定ファイルからトークンを取得
        let token = get_github_token().await?;
        // 例として固定のエディタバージョンを設定
        let editor_version = format!("Neovim/{}", "0.1.0");
        let http_client = reqwest::Client::new();

        Ok(Self {
            http_client,
            token,
            editor_version,
        })
    }

    /// API 呼び出しに必要なヘッダーを生成する
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        headers.insert(
            "Editor-Version",
            HeaderValue::from_str(&self.editor_version).unwrap(),
        );
        headers.insert(
            "Editor-Plugin-Version",
            HeaderValue::from_str("CopilotChat.nvim/*").unwrap(),
        );
        headers.insert(
            "Copilot-Integration-Id",
            HeaderValue::from_str("vscode-chat").unwrap(),
        );
        headers
    }

    /// ユーザーのプロンプトを送信して、Copilot Chat の応答を取得する
    async fn ask(&self, prompt: &str, model: &str) -> Result<ChatResponse, Box<dyn Error>> {
        let url = "https://api.githubcopilot.com/chat/completions";

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request_body = ChatRequest {
            model: model.to_string(),
            messages,
            temperature: 0.5,
            stream: false,
        };

        let response = self
            .http_client
            .post(url)
            .headers(self.get_headers())
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(format!("リクエスト失敗: {} - {}", status, text).into());
        }

        let chat_response = response.json::<ChatResponse>().await?;
        Ok(chat_response)
    }
}

#[tokio::main]
async fn main() {
    // クライアントの初期化
    let client = match CopilotClient::new().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("クライアント作成に失敗しました: {}", e);
            return;
        }
    };

    let prompt = "RustでCopilot Chatの機能を実装できますか？";
    // 使用するモデル ID を指定（実際のものに合わせてください）
    let model = "o1-xyz";

    match client.ask(prompt, model).await {
        Ok(response) => {
            for choice in response.choices {
                println!("回答: {}", choice.message.content);
                if let Some(reason) = choice.finish_reason {
                    println!("終了理由: {}", reason);
                }
            }
        }
        Err(e) => eprintln!("エラー: {}", e),
    }
}
