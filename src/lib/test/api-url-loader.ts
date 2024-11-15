import { setApiUrl } from "../api";

// Get the API URL from environment variables
const apiUrl = process.env.VITE_OPEN_SECRET_API_URL;

if (!apiUrl) {
  throw new Error("VITE_OPEN_SECRET_API_URL must be set in environment variables");
}

// Set the API URL before tests run
setApiUrl(apiUrl);

console.log("API URL set to:", apiUrl); 