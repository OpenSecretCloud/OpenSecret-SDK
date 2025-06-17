---
title: AI Integration
sidebar_position: 7
---

# AI Integration

OpenSecret provides a powerful way to integrate AI capabilities into your application while maintaining strong privacy guarantees. This guide explains how to use OpenSecret's AI features with end-to-end encryption.

## Overview

OpenSecret's AI integration:

- Encrypts data end-to-end from client to GPU
- Works with the OpenAI client library
- Supports streaming responses
- Maintains privacy guarantees through enclave technology
- Simplifies integrating AI into secure applications

## Prerequisites

Before using AI integration, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated (required for encryption)
3. You have installed the OpenAI client library

## Installation

Install the OpenAI client library:

```bash
# Using npm
npm install openai

# Using yarn
yarn add openai

# Using bun
bun add openai
```

## Basic AI Integration

### Setting Up the OpenAI Client

The OpenSecret SDK provides a custom fetch function (`aiCustomFetch`) that handles all the encryption and authentication for AI requests:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import OpenAI from "openai";

function AIComponent() {
  const os = useOpenSecret();
  const [openai, setOpenai] = useState<OpenAI | null>(null);
  
  // Initialize the OpenAI client when the user is authenticated
  useEffect(() => {
    if (os.auth.user && !openai) {
      const client = new OpenAI({
        baseURL: `${os.apiUrl}/v1/`,
        dangerouslyAllowBrowser: true,
        apiKey: "api-key-doesnt-matter", // Actual API key is handled by OpenSecret
        defaultHeaders: {
          "Accept-Encoding": "identity",
          "Content-Type": "application/json",
        },
        fetch: os.aiCustomFetch, // Use OpenSecret's encrypted fetch
      });
      
      setOpenai(client);
    }
  }, [os.auth.user, openai]);
  
  // Rest of your component...
}
```

### Simple AI Chat Example

```tsx
import { useState, FormEvent } from "react";
import { useOpenSecret } from "@opensecret/react";
import OpenAI from "openai";

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

function AIChat() {
  const os = useOpenSecret();
  const [query, setQuery] = useState("");
  const [response, setResponse] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    
    if (!query.trim() || loading || !os.auth.user) return;
    
    setLoading(true);
    setResponse("");
    setError("");
    
    try {
      const openai = new OpenAI({
        baseURL: `${os.apiUrl}/v1/`,
        dangerouslyAllowBrowser: true,
        apiKey: "api-key-doesnt-matter",
        defaultHeaders: {
          "Accept-Encoding": "identity",
          "Content-Type": "application/json",
        },
        fetch: os.aiCustomFetch,
      });
      
      // Choose the model to use
      const model = "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4";
      const messages = [{ role: "user", content: query } as ChatMessage];
      
      // Create a streaming request
      const stream = await openai.beta.chat.completions.stream({
        model,
        messages,
        stream: true,
      });
      
      // Process the streaming response
      let fullResponse = "";
      for await (const chunk of stream) {
        const content = chunk.choices[0]?.delta?.content || "";
        fullResponse += content;
        setResponse(fullResponse);
      }
      
      await stream.finalChatCompletion();
    } catch (error) {
      console.error("AI error:", error);
      setError(error instanceof Error ? error.message : "Failed to get AI response");
    } finally {
      setLoading(false);
    }
  }
  
  if (!os.auth.user) {
    return <div>Please log in to use the AI chat.</div>;
  }
  
  return (
    <div className="ai-chat">
      <h3>AI Chat</h3>
      
      {error && <div className="error">{error}</div>}
      
      <form onSubmit={handleSubmit} className="chat-form">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Enter your question..."
          disabled={loading}
        />
        <button type="submit" disabled={loading}>
          {loading ? "Sending..." : "Send"}
        </button>
      </form>
      
      {response && (
        <div className="response-container">
          <h4>Response:</h4>
          <div className="response-content">
            {response}
          </div>
        </div>
      )}
    </div>
  );
}
```

## Complete AI Chat Interface

Here's a more complete example with conversation history:

```tsx
import React, { useState, useEffect, useRef, FormEvent } from "react";
import { useOpenSecret } from "@opensecret/react";
import OpenAI from "openai";

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

function AIAssistant() {
  const os = useOpenSecret();
  const [openai, setOpenai] = useState<OpenAI | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  // Initialize OpenAI client when user is authenticated
  useEffect(() => {
    if (os.auth.user && !openai) {
      const client = new OpenAI({
        baseURL: `${os.apiUrl}/v1/`,
        dangerouslyAllowBrowser: true,
        apiKey: "api-key-doesnt-matter",
        defaultHeaders: {
          "Accept-Encoding": "identity",
          "Content-Type": "application/json",
        },
        fetch: os.aiCustomFetch,
      });
      
      setOpenai(client);
    }
  }, [os.auth.user, openai]);
  
  // Scroll to bottom of messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);
  
  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    
    if (!input.trim() || loading || !openai) return;
    
    // Add user message to chat
    const userMessage: ChatMessage = { role: "user", content: input };
    setMessages([...messages, userMessage]);
    setInput("");
    setLoading(true);
    setError("");
    
    // Start with empty assistant message that we'll build up
    const assistantMessage: ChatMessage = { role: "assistant", content: "" };
    setMessages([...messages, userMessage, assistantMessage]);
    
    try {
      // Choose the model to use
      const model = "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4";
      
      // Create a streaming request
      const stream = await openai.beta.chat.completions.stream({
        model,
        messages: [...messages, userMessage],
        stream: true,
      });
      
      // Process the streaming response
      for await (const chunk of stream) {
        const content = chunk.choices[0]?.delta?.content || "";
        
        if (content) {
          // Update the assistant's message with the new content
          assistantMessage.content += content;
          setMessages([...messages, userMessage, { ...assistantMessage }]);
        }
      }
      
      await stream.finalChatCompletion();
    } catch (error) {
      console.error("AI error:", error);
      setError(error instanceof Error ? error.message : "Failed to get AI response");
      
      // Remove the incomplete assistant message
      setMessages([...messages, userMessage]);
    } finally {
      setLoading(false);
    }
  }
  
  function clearChat() {
    setMessages([]);
  }
  
  if (!os.auth.user) {
    return <div className="login-message">Please log in to use the AI assistant.</div>;
  }
  
  return (
    <div className="ai-assistant">
      <div className="chat-header">
        <h3>AI Assistant</h3>
        <button onClick={clearChat} className="clear-button">
          Clear Chat
        </button>
      </div>
      
      {error && <div className="error">{error}</div>}
      
      <div className="messages-container">
        {messages.length === 0 ? (
          <div className="empty-chat">
            Send a message to start chatting with the AI assistant.
          </div>
        ) : (
          messages.map((msg, index) => (
            <div
              key={index}
              className={`message ${msg.role === "user" ? "user-message" : "assistant-message"}`}
            >
              <div className="message-role">
                {msg.role === "user" ? "You" : "Assistant"}:
              </div>
              <div className="message-content">{msg.content || "..."}</div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
      
      <form onSubmit={handleSubmit} className="input-form">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type your message..."
          disabled={loading}
        />
        <button type="submit" disabled={loading || !input.trim()}>
          {loading ? "Sending..." : "Send"}
        </button>
      </form>
    </div>
  );
}
```

## Using Custom AI Parameters

You can customize the AI parameters like temperature, max tokens, etc.:

```tsx
const stream = await openai.beta.chat.completions.stream({
  model: "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4",
  messages: [...messages, userMessage],
  stream: true,
  temperature: 0.7,
  max_tokens: 1000,
  top_p: 0.95,
  frequency_penalty: 0,
  presence_penalty: 0,
});
```

## Alternative Approach: Using aiCustomFetch Directly

If you need more control, you can use OpenSecret's `aiCustomFetch` directly:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

async function streamCompletion(prompt: string, handleContentUpdate: (content: string) => void) {
  const os = useOpenSecret();
  const customFetch = os.aiCustomFetch;
  let fullResponse = "";

  const response = await customFetch(`${os.apiUrl}/v1/chat/completions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "text/event-stream"
    },
    body: JSON.stringify({
      model: "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4",
      messages: [{ role: "user", content: prompt }],
      stream: true
    })
  });

  const reader = response.body?.getReader();
  if (!reader) throw new Error("Stream reader is not available");

  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line.startsWith("data: ")) {
        const data = line.slice(6);
        if (data === "[DONE]") break;
        
        try {
          const parsed = JSON.parse(data);
          const content = parsed.choices[0]?.delta?.content;
          if (content) {
            fullResponse += content;
            handleContentUpdate(fullResponse);
          }
        } catch (error) {
          console.error("Error parsing JSON:", error);
        }
      }
    }
  }

  return fullResponse;
}

function CustomAIComponent() {
  const [prompt, setPrompt] = useState("");
  const [response, setResponse] = useState("");
  const [loading, setLoading] = useState(false);
  
  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!prompt.trim() || loading) return;
    
    setLoading(true);
    setResponse("");
    
    try {
      await streamCompletion(prompt, (content) => {
        setResponse(content);
      });
    } catch (error) {
      console.error("AI error:", error);
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Enter your prompt..."
          disabled={loading}
        />
        <button type="submit" disabled={loading}>
          {loading ? "Processing..." : "Submit"}
        </button>
      </form>
      
      {response && (
        <div className="response">
          <h4>Response:</h4>
          <div>{response}</div>
        </div>
      )}
    </div>
  );
}
```

## How AI Encryption Works

The OpenSecret AI integration:

1. **Encrypts prompts client-side**: Your users' prompts are encrypted before leaving their browser
2. **Securely transmits encrypted data**: The encrypted data is sent to OpenSecret's secure enclaves
3. **Decrypts within the enclave**: Data is decrypted inside the secure enclave
4. **Processes with AI models**: The LLM processes the decrypted data
5. **Encrypts responses**: The AI's response is encrypted inside the enclave
6. **Returns encrypted data**: The encrypted response is sent back to the client
7. **Decrypts client-side**: The response is decrypted in the user's browser

This provides **true end-to-end encryption** for AI interactions.

## Available AI Models

OpenSecret currently supports the following models (check for the latest models available through the SDK):

- `hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4`
- `meta-llama/Llama-2-70b-chat-hf`
- And others based on availability

## Best Practices

1. **Check authentication**: Ensure the user is authenticated before using AI features
2. **Handle errors gracefully**: AI requests can fail for various reasons
3. **Provide feedback during loading**: Stream responses to show progress
4. **Sanitize user inputs**: Validate and clean inputs before sending to AI
5. **Implement rate limiting**: Add client-side rate limiting to prevent abuse

## Combining with Other OpenSecret Features

### Storing AI Conversations

You can combine AI with encrypted key-value storage to save conversations:

```tsx
// Store a conversation
async function saveConversation(messages: ChatMessage[]) {
  const os = useOpenSecret();
  const conversationId = Date.now().toString();
  await os.put(`conversation:${conversationId}`, JSON.stringify(messages));
  return conversationId;
}

// Retrieve a conversation
async function loadConversation(conversationId: string) {
  const os = useOpenSecret();
  const data = await os.get(`conversation:${conversationId}`);
  return data ? JSON.parse(data) as ChatMessage[] : [];
}

// List all conversations
async function listConversations() {
  const os = useOpenSecret();
  const items = await os.list();
  return items
    .filter(item => item.key.startsWith("conversation:"))
    .map(item => ({
      id: item.key.replace("conversation:", ""),
      updatedAt: new Date(item.updated_at)
    }));
}
```

### Encryption for Additional Privacy

For extremely sensitive applications, you can add an additional layer of encryption:

```tsx
// Encrypt AI prompt before sending
async function encryptedAIRequest(prompt: string) {
  const os = useOpenSecret();
  
  // First encrypt with user's key
  const { encrypted_data } = await os.encryptData(prompt);
  
  // Then send the encrypted prompt to AI
  // and tell it to decrypt the prompt first
  const response = await os.aiRequest({
    message: `This is an encrypted message: ${encrypted_data}. 
    Please decrypt it first, then respond to the decrypted content.`
  });
  
  return response;
}
```

## Security Considerations

1. **Authentication is required**: The user must be authenticated to use AI features
2. **Client-side security**: Ensure sensitive context isn't leaked in UI
3. **Enclave limitations**: While secure, enclaves have specific security boundaries
4. **Data residency**: Understand where data is processed for compliance purposes

## Document-Based AI Chat

You can combine document upload with AI chat for intelligent document analysis:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import OpenAI from "openai";

function DocumentAIChat() {
  const os = useOpenSecret();
  const [documentContext, setDocumentContext] = useState("");
  const [fileName, setFileName] = useState("");
  
  async function handleDocumentUpload(file: File) {
    try {
      // Upload and extract text from document
      const result = await os.uploadDocument(file);
      setDocumentContext(result.text);
      setFileName(result.filename);
      
      // Now you can use the document text as context for AI chat
      return result;
    } catch (error) {
      console.error("Document upload failed:", error);
      throw error;
    }
  }
  
  async function askQuestionAboutDocument(question: string) {
    if (!documentContext || !os.auth.user) return;
    
    const openai = new OpenAI({
      baseURL: `${os.apiUrl}/v1/`,
      dangerouslyAllowBrowser: true,
      apiKey: "api-key-doesnt-matter",
      fetch: os.aiCustomFetch,
    });
    
    const messages = [
      {
        role: "system" as const,
        content: "You are a helpful assistant that answers questions based on the provided document."
      },
      {
        role: "user" as const,
        content: `Document: ${fileName}\n\n${documentContext}\n\nQuestion: ${question}`
      }
    ];
    
    const stream = await openai.beta.chat.completions.stream({
      model: "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4",
      messages,
      stream: true,
    });
    
    let response = "";
    for await (const chunk of stream) {
      const content = chunk.choices[0]?.delta?.content || "";
      response += content;
    }
    
    return response;
  }
  
  // Rest of your component...
}
```

For a complete example of document-based AI chat, see the [Document Upload Guide](./document-upload).

## What's Next

- [Document Upload](./document-upload) - Upload and process documents for AI analysis
- [Remote Attestation](./remote-attestation) - Learn how OpenSecret verifies its enclave security
- [Key-Value Storage](./key-value-storage) - Store AI conversations and context securely
- [Data Encryption](./data-encryption) - Add additional encryption layers