import { setPlatformApiUrl } from "../platformApi";

// Get the API URL from environment variables
const apiUrl = process.env.VITE_OPEN_SECRET_API_URL;

if (!apiUrl) {
  throw new Error("VITE_OPEN_SECRET_API_URL must be set in environment variables");
}

// Set the Platform API URL before tests run
setPlatformApiUrl(apiUrl);

console.log("Platform API URL set to:", apiUrl);
