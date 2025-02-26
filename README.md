# Copilot Client Library

A Rust client for interacting with the GitHub Copilot API. This library simplifies accessing the Copilot API by handling token retrieval, model and agent fetching, chat completions, and embeddings generation. It leverages [reqwest](https://crates.io/crates/reqwest) for HTTP communication and [serde](https://crates.io/crates/serde) for JSON serialization and deserialization.

---

## Features

- **GitHub Token Retrieval:** Automatically obtains a GitHub token from environment variables or configuration files.
- **Model & Agent Fetching:** Retrieve available Copilot models and agent information.
- **Chat Completions:** Send chat requests and receive model-generated responses.
- **Embeddings:** Generate embeddings for input texts.
- **Async/Await Support:** Built using asynchronous Rust with the [tokio](https://crates.io/crates/tokio) runtime.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
copilot_client = "0.1.0"
```

---

## Usage

Ensure you have your GitHub token available via the `GITHUB_TOKEN` environment variable or in the appropriate configuration file. The token is required for authenticating with the GitHub Copilot API.

Below is a sample application demonstrating how to use the client:

```rust
use std::error::Error;
use copilot_client::{CopilotClient, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Define the editor version. This is used in API requests.
    let editor_version = "Neovim/0.9.0".to_string();

    // Create a new CopilotClient by retrieving the GitHub token from environment variables
    // or configuration files, and fetching the list of available models.
    let client = CopilotClient::from_env_with_models(editor_version).await?;

    // Retrieve and display the list of agents.
    let agents = client.get_agents().await?;
    println!("Agents: {agents:?}");

    // Retrieve and display the available models (the internal model list maintained by the client).
    let models = client.get_models().await?;
    println!("Available models: {models:?}");

    // Example chat request:
    // The system message instructs the assistant to behave as a highly skilled helper,
    // and the user asks a question in English.
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a highly skilled assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Can you explain how to send an HTTP request in Rust?".to_string(),
        },
    ];

    // If the specified model ID is not found in the client's internal model list,
    // an error will be returned.
    let model_id = "gemini-2.0-flash-001".to_string();
    match client.chat_completion(messages, model_id).await {
        Ok(chat_response) => println!("Chat Response: {chat_response:?}"),
        Err(e) => println!("Chat completion request error: {e}"),
    }

    // Example embeddings request: generate embeddings for the input string.
    let inputs = vec!["Rust programming language".to_string()];
    let embeddings = client.get_embeddings(inputs).await?;
    println!("Embeddings: {embeddings:?}");

    Ok(())
}
```

---

## Configuration

### GitHub Token

The client retrieves the GitHub token from the environment variable `GITHUB_TOKEN`. Alternatively, if you are running in an environment such as Codespaces or have your token stored in one of the configuration files (`hosts.json` or `apps.json` under your configuration directory), the client will attempt to read the token from there.

### Configuration Directory

- **Unix:** Uses `XDG_CONFIG_HOME` or defaults to `$HOME/.config`.
- **Windows:** Uses `LOCALAPPDATA`.

---

## Error Handling

The library defines a custom error type, [`CopilotError`](src/lib.rs), which encompasses errors related to invalid models, token retrieval, HTTP issues, and other miscellaneous errors. Ensure you handle these errors gracefully in your application.

---

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check [issues](https://github.com/yourusername/copilot_client/issues) if you want to contribute.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- [Reqwest](https://crates.io/crates/reqwest) for HTTP client functionality.
- [Serde](https://crates.io/crates/serde) for seamless JSON serialization and deserialization.
- [Tokio](https://crates.io/crates/tokio) for asynchronous runtime support.

---
