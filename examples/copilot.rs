use copilot_client::{CopilotClient, Message};
use std::error::Error;

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
    // In this sample, the system prompt tells the assistant it is highly capable,
    // and the user asks: "Can you explain how to send an HTTP request in Rust?"
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
