import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { OpenSecretProvider } from "./lib/main.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <OpenSecretProvider config={{ apiUrl: import.meta.env.VITE_OPEN_SECRET_API_URL }}>
      <App />
    </OpenSecretProvider>
  </StrictMode>
);
