import { useOpenSecret } from "./lib";
import { useState, FormEvent } from "react";
import OpenAI from "openai";

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

export function AI() {
  const os = useOpenSecret();
  const [query, setQuery] = useState("");
  const [response, setResponse] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    setLoading(true);
    setResponse("");

    if (!os.auth.user) {
        alert("Please log in to use the AI chat.");
        return;
    }

    const customFetch = os.aiCustomFetch;

    if (!query.trim() || loading || !customFetch) return;

    try {
      console.log("Starting chat request to URL:", `${os.apiUrl}/v1/`);
      
      const openai = new OpenAI({
        baseURL: `${os.apiUrl}/v1/`,
        dangerouslyAllowBrowser: true,
        apiKey: "api-key-doesnt-matter",
        defaultHeaders: {
          "Accept-Encoding": "identity",
          "Content-Type": "application/json",
        },
        fetch: customFetch,
      });

      console.log("Created OpenAI client");

      const model = "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4";
      const messages = [{ role: "user", content: query } as ChatMessage];

      console.log("Starting stream request");
      const stream = await openai.beta.chat.completions.stream({
        model,
        messages,
        stream: true,
      });

      console.log("Stream created successfully");

      let fullResponse = "";
      for await (const chunk of stream) {
        console.log("Received chunk:", chunk);
        const content = chunk.choices[0]?.delta?.content || "";
        fullResponse += content;
        setResponse(fullResponse);
      }

      await stream.finalChatCompletion();
    } catch (error) {
      console.error("Chat error:", error);
      if (error instanceof Error) {
        console.error("Error details:", {
          message: error.message,
          stack: error.stack,
          name: error.name
        });
      }
      setResponse("Error: Failed to get response. " + (error as Error).message);
    } finally {
      setLoading(false);
    }
  };

  if (!os.auth.user) {
    return <div>Please log in to use the AI chat.</div>;
  }

  return (
    <div className="p-4">
      <form onSubmit={handleSubmit} className="mb-4">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Enter your question..."
          className="w-full p-2 border rounded"
          disabled={loading}
        />
        <button 
          type="submit" 
          disabled={loading}
          className="mt-2 px-4 py-2 bg-blue-500 text-white rounded disabled:bg-gray-400"
        >
          {loading ? "Loading..." : "Send"}
        </button>
      </form>

      {response && (
        <div className="mt-4 p-4 border rounded bg-gray-50 whitespace-pre-wrap">
          {response}
        </div>
      )}
    </div>
  );
}
