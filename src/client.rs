use std::{env, error::Error, fs, path::Path};

use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client as HttpClient,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// GitHub Copilot のトークンレスポンス（`expires_at` は Unix タイムスタンプ）
#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotTokenResponse {
    pub token: String,
    pub expires_at: u64,
}

/// エージェント情報
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// エージェント取得レスポンス (GET <https://api.githubcopilot.com/agents>)
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentsResponse {
    pub agents: Vec<Agent>,
}

/// モデル情報
#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub tokenizer: Option<String>,
    pub max_input_tokens: Option<u32>,
    pub max_output_tokens: Option<u32>,
}

/// モデル取得レスポンス (GET <https://api.githubcopilot.com/models>)
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

/// メッセージ情報（role には "system", "user", "assistant" など）
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// チャットリクエストの構造体 (POST <https://api.githubcopilot.com/chat/completions>)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub n: u32,
    pub top_p: f64,
    pub stream: bool,
    pub temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// チャットレスポンス内の選択肢情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: Message,
    pub finish_reason: Option<String>,
    pub usage: Option<TokenUsage>,
}

/// トークン使用量情報
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub total_tokens: u32,
}

/// チャットレスポンス全体の構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

/// 埋め込みリクエストの構造体 (POST <https://api.githubcopilot.com/embeddings>)
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub dimensions: u32,
    pub input: Vec<String>,
    pub model: String,
}

/// 埋め込み情報の構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Embedding {
    pub index: usize,
    pub embedding: Vec<f64>,
}

/// 埋め込みレスポンスの構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<Embedding>,
}

/// GitHub Copilot クライアント
pub struct CopilotClient {
    http_client: HttpClient,
    github_token: String,
    editor_version: String,
}

impl CopilotClient {
    /// `from_env()` を利用すると、内部で GitHub トークン取得処理（環境変数または設定ファイルから）を行います。
    pub fn from_env(editor_version: String) -> Result<Self, Box<dyn Error>> {
        let github_token = get_github_token()?;
        Ok(Self::new(github_token, editor_version))
    }

    /// 指定した GitHub トークンとエディタバージョンから新しいクライアントを生成します。
    pub fn new(github_token: String, editor_version: String) -> Self {
        let http_client = HttpClient::new();
        CopilotClient {
            http_client,
            github_token,
            editor_version,
        }
    }

    /// GitHub Copilot 用の認証ヘッダーを生成します。Lua 版と同様に、内部で取得したトークンを `"Bearer ..."` としてセットし、`Editor-Version`、`Editor-Plugin-Version`、`Copilot-Integration-Id` なども付与します。
    async fn get_headers(&self) -> Result<HeaderMap, Box<dyn Error>> {
        let token = self.get_copilot_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        headers.insert(
            "Editor-Version",
            HeaderValue::from_str(&self.editor_version)?,
        );
        headers.insert(
            "Editor-Plugin-Version",
            HeaderValue::from_static("CopilotChat.nvim/*"),
        );
        headers.insert(
            "Copilot-Integration-Id",
            HeaderValue::from_static("vscode-chat"),
        );
        // GitHub API では User-Agent および Accept ヘッダーが必須です
        headers.insert(USER_AGENT, HeaderValue::from_static("CopilotChat.nvim"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// GitHub Copilot のトークンを取得します。Lua の実装と同様、<https://api.github.com/copilot_internal/v2/token> に対して、環境変数または設定ファイルから取得した GitHub トークンを使ってリクエストします。
    async fn get_copilot_token(&self) -> Result<String, Box<dyn Error>> {
        let url = "https://api.github.com/copilot_internal/v2/token";
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("CopilotChat.nvim"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Token {}", self.github_token))?,
        );
        let res = self
            .http_client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        let token_response: CopilotTokenResponse = res.json().await?;
        Ok(token_response.token)
    }

    /// エージェント情報を取得します (GET <https://api.githubcopilot.com/agents>)。
    pub async fn get_agents(&self) -> Result<Vec<Agent>, Box<dyn Error>> {
        let url = "https://api.githubcopilot.com/agents";
        let headers = self.get_headers().await?;
        let res = self
            .http_client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        let agents_response: AgentsResponse = res.json().await?;
        Ok(agents_response.agents)
    }

    /// モデル情報を取得します (GET <https://api.githubcopilot.com/models>)。
    pub async fn get_models(&self) -> Result<Vec<Model>, Box<dyn Error>> {
        let url = "https://api.githubcopilot.com/models";
        let headers = self.get_headers().await?;
        let res = self
            .http_client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        let models_response: ModelsResponse = res.json().await?;
        Ok(models_response.data)
    }

    /// チャット補完リクエストを送信します (POST <https://api.githubcopilot.com/chat/completions>)。
    /// `messages` にはシステム、ユーザー、アシスタントの各メッセージを含めます。
    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, Box<dyn Error>> {
        let url = "https://api.githubcopilot.com/chat/completions";
        let headers = self.get_headers().await?;
        let request_body = ChatRequest {
            model: model_id, // 必要に応じてモデル ID を指定してください
            messages,
            n: 1,
            top_p: 1.0,
            stream: false,
            temperature: 0.5,
            max_tokens: None,
        };
        let res = self
            .http_client
            .post(url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        let chat_response: ChatResponse = res.json().await?;
        Ok(chat_response)
    }

    /// 埋め込み生成リクエストを送信します (POST <https://api.githubcopilot.com/embeddings>)。
    pub async fn get_embeddings(
        &self,
        inputs: Vec<String>,
    ) -> Result<Vec<Embedding>, Box<dyn Error>> {
        let url = "https://api.githubcopilot.com/embeddings";
        let headers = self.get_headers().await?;
        let request_body = EmbeddingRequest {
            dimensions: 512,
            input: inputs,
            model: "text-embedding-3-small".to_string(),
        };
        let res = self
            .http_client
            .post(url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        let embedding_response: EmbeddingResponse = res.json().await?;
        Ok(embedding_response.data)
    }
}

/// `get_github_token()` は、まず環境変数 `<GITHUB_TOKEN>` と "CODESPACES" からトークンを取得し、
/// 存在しなければユーザーの設定ディレクトリ内の `github-copilot/hosts.json` または `github-copilot/apps.json` を
/// 読み込み、"github.com" を含むキーの `oauth_token` を返します。
pub fn get_github_token() -> Result<String, Box<dyn Error>> {
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if env::var("CODESPACES").is_ok() {
            return Ok(token);
        }
    }
    let config_dir = get_config_path()?;
    let file_paths = vec![
        format!("{config_dir}/github-copilot/hosts.json"),
        format!("{config_dir}/github-copilot/apps.json"),
    ];
    for file_path in file_paths {
        if Path::new(&file_path).exists() {
            let content = fs::read_to_string(&file_path)?;
            let json_value: Value = serde_json::from_str(&content)?;
            if let Some(obj) = json_value.as_object() {
                for (key, value) in obj {
                    if key.contains("github.com") {
                        if let Some(oauth_token) = value.get("oauth_token") {
                            if let Some(token_str) = oauth_token.as_str() {
                                return Ok(token_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    Err("Failed to find GitHub token".into())
}

/// ユーザーの設定ディレクトリを返します。まず `<XDG_CONFIG_HOME>` を、なければ `$HOME/.config` を返します。
pub fn get_config_path() -> Result<String, Box<dyn Error>> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Ok(xdg);
        }
    }
    if cfg!(target_os = "windows") {
        if let Ok(local) = env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                return Ok(local);
            }
        }
    } else if let Ok(home) = env::var("HOME") {
        return Ok(format!("{home}/.config"));
    }
    Err("Failed to find config directory".into())
}
