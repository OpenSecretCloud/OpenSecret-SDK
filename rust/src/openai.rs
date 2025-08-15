// OpenAI API implementation
// TODO: Implement after testing attestation

use crate::{client::OpenSecretClient, error::Result, types::*};

impl OpenSecretClient {
    pub async fn create_chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        todo!("Implement after attestation testing")
    }
}