---
title: Document Upload
sidebar_position: 8
---

# Document Upload

OpenSecret provides a secure document upload and text extraction service that allows you to process various document formats while maintaining end-to-end encryption. This guide explains how to use the document upload API to extract text from documents for use in AI prompts or other applications.

## Overview

The document upload feature:

- Extracts text from various document formats (PDF, DOCX, TXT, etc.)
- Maintains end-to-end encryption using session keys
- Enforces a 10MB file size limit
- Requires JWT authentication (guest users are not supported)
- Includes usage limit enforcement
- Returns extracted text ready for use in chat prompts
- Supports asynchronous processing with status polling

## Upload Methods

OpenSecret SDK provides three methods for document upload:

1. **`uploadDocument`** - Initiates upload and returns a task ID immediately
2. **`checkDocumentStatus`** - Checks the status of a processing task
3. **`uploadDocumentWithPolling`** - Convenient wrapper that handles polling automatically

## Prerequisites

Before using document upload, ensure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated (not a guest user)
3. You have a valid session established

## Basic Document Upload

### Method 1: Upload with Automatic Polling (Recommended)

The simplest way to upload a document is using `uploadDocumentWithPolling`, which handles the async processing automatically:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function DocumentUploader() {
  const os = useOpenSecret();
  const [file, setFile] = useState<File | null>(null);
  const [extractedText, setExtractedText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleUpload() {
    if (!file || loading || !os.auth.user) return;

    setLoading(true);
    setError("");

    try {
      // Upload and process the document with automatic polling
      const result = await os.uploadDocumentWithPolling(file, {
        onProgress: (status, progress) => {
          console.log(`Status: ${status}, Progress: ${progress || 0}%`);
        }
      });
      
      // The result contains the extracted text
      setExtractedText(result.text);
      
      console.log(`Extracted ${result.size} bytes from ${result.filename}`);
    } catch (error) {
      console.error("Upload error:", error);
      if (error instanceof Error) {
        if (error.message.includes("10MB")) {
          setError("File is too large. Maximum size is 10MB.");
        } else if (error.message.includes("401")) {
          setError("Guest users cannot upload documents. Please create an account.");
        } else if (error.message.includes("403")) {
          setError("Usage limit exceeded. Please try again later.");
        } else {
          setError(error.message || "Failed to upload document");
        }
      }
    } finally {
      setLoading(false);
    }
  }

  if (!os.auth.user) {
    return <div>Please log in to upload documents.</div>;
  }

  return (
    <div className="document-uploader">
      <h3>Document Upload</h3>
      
      {error && <div className="error">{error}</div>}
      
      <input
        type="file"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
        accept=".pdf,.docx,.doc,.txt,.rtf"
        disabled={loading}
      />
      
      <button 
        onClick={handleUpload} 
        disabled={!file || loading}
      >
        {loading ? "Processing..." : "Upload & Extract Text"}
      </button>
      
      {extractedText && (
        <div className="extracted-text">
          <h4>Extracted Text:</h4>
          <pre>{extractedText}</pre>
        </div>
      )}
    </div>
  );
}
```

### Method 2: Manual Status Polling

For more control over the polling process, you can use `uploadDocument` and `checkDocumentStatus` separately:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function DocumentUploaderManual() {
  const os = useOpenSecret();
  const [file, setFile] = useState<File | null>(null);
  const [taskId, setTaskId] = useState<string>("");
  const [status, setStatus] = useState<string>("");
  const [extractedText, setExtractedText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleUpload() {
    if (!file || loading || !os.auth.user) return;

    setLoading(true);
    setError("");
    setStatus("uploading");

    try {
      // Step 1: Upload the document and get task ID
      const initResponse = await os.uploadDocument(file);
      setTaskId(initResponse.task_id);
      setStatus("pending");
      
      // Step 2: Poll for status
      let attempts = 0;
      const maxAttempts = 150; // 5 minutes with 2s interval
      
      while (attempts < maxAttempts) {
        const statusResponse = await os.checkDocumentStatus(initResponse.task_id);
        setStatus(statusResponse.status);
        
        if (statusResponse.status === "success") {
          if (statusResponse.document) {
            setExtractedText(statusResponse.document.text);
            console.log(`Extracted ${statusResponse.document.size} bytes`);
          }
          break;
        } else if (statusResponse.status === "failure") {
          throw new Error(statusResponse.error || "Processing failed");
        }
        
        // Wait before next poll
        await new Promise(resolve => setTimeout(resolve, 2000));
        attempts++;
      }
      
      if (attempts >= maxAttempts) {
        throw new Error("Document processing timed out");
      }
    } catch (error) {
      console.error("Upload error:", error);
      setError(error instanceof Error ? error.message : "Failed to process document");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="document-uploader">
      <h3>Document Upload (Manual Polling)</h3>
      
      {error && <div className="error">{error}</div>}
      {taskId && <div>Task ID: {taskId}</div>}
      {status && <div>Status: {status}</div>}
      
      <input
        type="file"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
        accept=".pdf,.docx,.doc,.txt,.rtf"
        disabled={loading}
      />
      
      <button 
        onClick={handleUpload} 
        disabled={!file || loading}
      >
        {loading ? `Processing (${status})...` : "Upload & Extract Text"}
      </button>
      
      {extractedText && (
        <div className="extracted-text">
          <h4>Extracted Text:</h4>
          <pre>{extractedText}</pre>
        </div>
      )}
    </div>
  );
}
```

### Method 3: Upload with Progress Tracking

Use the `onProgress` callback to show detailed progress to users:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function DocumentUploaderWithProgress() {
  const os = useOpenSecret();
  const [file, setFile] = useState<File | null>(null);
  const [progress, setProgress] = useState<{ status: string; percent: number }>({
    status: "",
    percent: 0
  });
  const [extractedText, setExtractedText] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleUpload() {
    if (!file || loading || !os.auth.user) return;

    setLoading(true);
    setProgress({ status: "uploading", percent: 0 });

    try {
      const result = await os.uploadDocumentWithPolling(file, {
        pollInterval: 1000, // Check every second
        maxAttempts: 300,   // 5 minutes total
        onProgress: (status, progressPercent) => {
          setProgress({
            status,
            percent: progressPercent || (status === "started" ? 50 : 0)
          });
        }
      });
      
      setExtractedText(result.text);
      setProgress({ status: "completed", percent: 100 });
    } catch (error) {
      console.error("Upload error:", error);
      setProgress({ status: "failed", percent: 0 });
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="document-uploader">
      <h3>Document Upload with Progress</h3>
      
      <input
        type="file"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
        accept=".pdf,.docx,.doc,.txt,.rtf"
        disabled={loading}
      />
      
      <button 
        onClick={handleUpload} 
        disabled={!file || loading}
      >
        Upload & Extract Text
      </button>
      
      {loading && (
        <div className="progress">
          <div>Status: {progress.status}</div>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${progress.percent}%` }}
            />
          </div>
        </div>
      )}
      
      {extractedText && (
        <div className="extracted-text">
          <h4>Extracted Text:</h4>
          <pre>{extractedText}</pre>
        </div>
      )}
    </div>
  );
}
```

## Document Q&A Integration

Combine document upload with AI chat for intelligent document analysis:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import OpenAI from "openai";

function DocumentQA() {
  const os = useOpenSecret();
  const [file, setFile] = useState<File | null>(null);
  const [documentText, setDocumentText] = useState("");
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState("");
  const [loading, setLoading] = useState(false);
  const [uploadLoading, setUploadLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleFileUpload() {
    if (!file || uploadLoading || !os.auth.user) return;

    setUploadLoading(true);
    setError("");

    try {
      const result = await os.uploadDocumentWithPolling(file);
      setDocumentText(result.text);
    } catch (error) {
      console.error("Upload error:", error);
      setError(error instanceof Error ? error.message : "Failed to upload document");
    } finally {
      setUploadLoading(false);
    }
  }

  async function handleQuestionSubmit(e: React.FormEvent) {
    e.preventDefault();
    
    if (!question.trim() || !documentText || loading || !os.auth.user) return;

    setLoading(true);
    setAnswer("");
    setError("");

    try {
      // Initialize OpenAI client
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

      // Create a prompt with the document context
      const messages = [
        {
          role: "system" as const,
          content: "You are a helpful assistant that answers questions based on the provided document. Only use information from the document to answer questions."
        },
        {
          role: "user" as const,
          content: `Here is a document:\n\n${documentText}\n\nBased on this document, please answer the following question: ${question}`
        }
      ];

      // Get AI response
      const stream = await openai.beta.chat.completions.stream({
        model: "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4",
        messages,
        stream: true,
        temperature: 0.3, // Lower temperature for more factual responses
      });

      // Process streaming response
      let fullAnswer = "";
      for await (const chunk of stream) {
        const content = chunk.choices[0]?.delta?.content || "";
        fullAnswer += content;
        setAnswer(fullAnswer);
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
    return <div>Please log in to use Document Q&A.</div>;
  }

  return (
    <div className="document-qa">
      <h3>Document Q&A</h3>
      
      {error && <div className="error">{error}</div>}
      
      {!documentText ? (
        <div className="upload-section">
          <h4>Step 1: Upload a Document</h4>
          <input
            type="file"
            onChange={(e) => setFile(e.target.files?.[0] || null)}
            accept=".pdf,.docx,.doc,.txt,.rtf"
            disabled={uploadLoading}
          />
          <button 
            onClick={handleFileUpload} 
            disabled={!file || uploadLoading}
          >
            {uploadLoading ? "Processing..." : "Upload Document"}
          </button>
        </div>
      ) : (
        <div className="qa-section">
          <h4>Step 2: Ask Questions</h4>
          <div className="document-info">
            Document loaded: {file?.name} ({documentText.length} characters)
          </div>
          
          <form onSubmit={handleQuestionSubmit}>
            <input
              type="text"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="Ask a question about the document..."
              disabled={loading}
            />
            <button type="submit" disabled={loading || !question.trim()}>
              {loading ? "Thinking..." : "Ask"}
            </button>
          </form>
          
          {answer && (
            <div className="answer-section">
              <h4>Answer:</h4>
              <div className="answer-content">{answer}</div>
            </div>
          )}
          
          <button 
            onClick={() => {
              setDocumentText("");
              setFile(null);
              setQuestion("");
              setAnswer("");
            }}
            className="reset-button"
          >
            Upload a Different Document
          </button>
        </div>
      )}
    </div>
  );
}
```

## Advanced Document Processing

### Multiple Document Analysis

Process multiple documents and analyze them together:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

type ProcessedDocument = {
  filename: string;
  text: string;
  size: number;
};

function MultiDocumentProcessor() {
  const os = useOpenSecret();
  const [documents, setDocuments] = useState<ProcessedDocument[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(e.target.files || []);
    if (files.length === 0 || loading || !os.auth.user) return;

    setLoading(true);
    setError("");

    try {
      // Process files in parallel with polling
      const uploadPromises = files.map(async (file) => {
        try {
          const result = await os.uploadDocumentWithPolling(file);
          return result;
        } catch (error) {
          console.error(`Failed to upload ${file.name}:`, error);
          return null;
        }
      });

      const results = await Promise.all(uploadPromises);
      const successfulUploads = results.filter((r): r is ProcessedDocument => r !== null);
      
      setDocuments([...documents, ...successfulUploads]);
      
      if (successfulUploads.length < files.length) {
        setError(`Successfully uploaded ${successfulUploads.length} of ${files.length} files`);
      }
    } catch (error) {
      console.error("Upload error:", error);
      setError(error instanceof Error ? error.message : "Failed to upload documents");
    } finally {
      setLoading(false);
    }
  }

  async function analyzeDocuments() {
    if (documents.length === 0 || !os.auth.user) return;

    // Combine all document texts
    const combinedText = documents
      .map((doc) => `--- Document: ${doc.filename} ---\n${doc.text}\n`)
      .join("\n\n");

    // You can now use combinedText with AI for analysis
    console.log("Combined text length:", combinedText.length);
    
    // Example: Find common themes, summarize multiple documents, etc.
  }

  return (
    <div className="multi-document-processor">
      <h3>Multi-Document Analysis</h3>
      
      {error && <div className="error">{error}</div>}
      
      <input
        type="file"
        multiple
        onChange={handleFileSelect}
        accept=".pdf,.docx,.doc,.txt,.rtf"
        disabled={loading}
      />
      
      {loading && <div>Processing documents...</div>}
      
      {documents.length > 0 && (
        <div className="documents-list">
          <h4>Uploaded Documents ({documents.length})</h4>
          <ul>
            {documents.map((doc, index) => (
              <li key={index}>
                {doc.filename} - {doc.text.length} characters
              </li>
            ))}
          </ul>
          
          <button onClick={analyzeDocuments}>
            Analyze All Documents
          </button>
        </div>
      )}
    </div>
  );
}
```

### Document Storage

Store extracted document text for later use:

```tsx
async function saveDocument(document: ProcessedDocument) {
  const os = useOpenSecret();
  const docId = `doc:${Date.now()}:${document.filename}`;
  
  // Store document metadata and text
  await os.put(docId, JSON.stringify({
    filename: document.filename,
    text: document.text,
    size: document.size,
    uploadedAt: new Date().toISOString()
  }));
  
  return docId;
}

async function loadDocument(docId: string) {
  const os = useOpenSecret();
  const data = await os.get(docId);
  return data ? JSON.parse(data) : null;
}

async function listSavedDocuments() {
  const os = useOpenSecret();
  const items = await os.list();
  
  return items
    .filter(item => item.key.startsWith("doc:"))
    .map(item => ({
      id: item.key,
      uploadedAt: new Date(item.created_at)
    }));
}
```

## File Size Validation

The SDK automatically validates file size, but you can add client-side validation for better UX:

```tsx
function DocumentUploadWithValidation() {
  const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState("");

  function handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    const selectedFile = e.target.files?.[0];
    
    if (!selectedFile) {
      setFile(null);
      return;
    }

    // Client-side file size validation
    if (selectedFile.size > MAX_FILE_SIZE) {
      setError(`File is too large. Maximum size is ${MAX_FILE_SIZE / 1024 / 1024}MB.`);
      setFile(null);
      e.target.value = ""; // Clear the input
      return;
    }

    setError("");
    setFile(selectedFile);
  }

  return (
    <div>
      {error && <div className="error">{error}</div>}
      
      <input
        type="file"
        onChange={handleFileSelect}
        accept=".pdf,.docx,.doc,.txt,.rtf"
      />
      
      {file && (
        <div className="file-info">
          Selected: {file.name} ({(file.size / 1024 / 1024).toFixed(2)}MB)
        </div>
      )}
    </div>
  );
}
```

## Supported Document Formats

The Tinfoil document processing service supports common document formats including:

- **PDF** (.pdf) - Portable Document Format
- **Microsoft Word** (.docx, .doc) - Word documents
- **Plain Text** (.txt) - Simple text files
- **Rich Text Format** (.rtf) - Formatted text documents
- **Microsoft Excel** (.xlsx, .xls) - Spreadsheets (text content only)
- **Microsoft PowerPoint** (.pptx, .ppt) - Presentations (text content only)

## Error Handling

Handle different error scenarios appropriately:

```tsx
async function uploadWithErrorHandling(file: File) {
  const os = useOpenSecret();
  
  try {
    const result = await os.uploadDocumentWithPolling(file);
    return { success: true, data: result };
  } catch (error) {
    if (error instanceof Error) {
      // File too large
      if (error.message.includes("exceeds maximum limit")) {
        return { 
          success: false, 
          error: "File is too large. Please upload a file smaller than 10MB." 
        };
      }
      
      // Authentication issues
      if (error.message.includes("401") || error.message.includes("not authenticated")) {
        return { 
          success: false, 
          error: "Please sign in to upload documents. Guest accounts are not supported." 
        };
      }
      
      // Usage limits
      if (error.message.includes("403") || error.message.includes("limit")) {
        return { 
          success: false, 
          error: "Usage limit reached. Please try again later or upgrade your plan." 
        };
      }
      
      // Processing errors
      if (error.message.includes("500") || error.message.includes("processing")) {
        return { 
          success: false, 
          error: "Failed to process document. The file may be corrupted or in an unsupported format." 
        };
      }
    }
    
    // Generic error
    return { 
      success: false, 
      error: "An unexpected error occurred. Please try again." 
    };
  }
}
```

## API Reference

### uploadDocument

```typescript
uploadDocument(file: File | Blob): Promise<DocumentUploadInitResponse>
```

Initiates document upload and returns immediately with a task ID.

**Returns:**
```typescript
{
  task_id: string;    // Unique identifier for the processing task
  filename: string;   // Name of the uploaded file
  size: number;       // Size of the file in bytes
}
```

### checkDocumentStatus

```typescript
checkDocumentStatus(taskId: string): Promise<DocumentStatusResponse>
```

Checks the status of a document processing task.

**Returns:**
```typescript
{
  status: "pending" | "started" | "success" | "failure";
  progress?: number;           // Optional progress percentage
  error?: string;              // Error message if status is "failure"
  document?: DocumentResponse; // Processed document if status is "success"
}
```

### uploadDocumentWithPolling

```typescript
uploadDocumentWithPolling(
  file: File | Blob,
  options?: {
    pollInterval?: number;    // Time between polls in ms (default: 2000)
    maxAttempts?: number;     // Max polling attempts (default: 150)
    onProgress?: (status: string, progress?: number) => void;
  }
): Promise<DocumentResponse>
```

Convenience method that uploads a document and automatically polls until completion.

**Returns:**
```typescript
{
  text: string;      // Extracted text content
  filename: string;  // Original filename
  size: number;      // Size in bytes
}
```

## Security Considerations

1. **Authentication Required**: Only authenticated users can upload documents
2. **Guest Users Blocked**: Guest accounts receive a 401 error
3. **End-to-End Encryption**: All uploads and responses are encrypted using session keys
4. **File Size Limits**: 10MB limit prevents abuse and ensures performance
5. **Usage Limits**: API enforces usage limits to prevent abuse
6. **Async Processing**: Documents are processed asynchronously to handle large files reliably

## Best Practices

1. **Validate File Types**: Check file extensions before upload
2. **Show Progress**: Provide visual feedback during upload and processing
3. **Handle Large Files**: Inform users about the 10MB limit upfront
4. **Cache Results**: Store extracted text to avoid re-uploading the same document
5. **Error Recovery**: Provide clear error messages and recovery options

## What's Next

- [AI Integration](./ai-integration) - Use extracted text with AI for intelligent document analysis
- [Key-Value Storage](./key-value-storage) - Store document text and metadata securely
- [Data Encryption](./data-encryption) - Add additional encryption layers for sensitive documents