pub mod adapter;
pub mod ollama;
pub mod openai_compat;

pub use adapter::chat_with_config;
pub use ollama::OllamaAdapter;
