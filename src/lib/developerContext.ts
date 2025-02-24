import { useContext } from "react";
import { OpenSecretDeveloperContext, OpenSecretDeveloperContextType } from "./developer";

export function useOpenSecretDeveloper(): OpenSecretDeveloperContextType {
  const context = useContext(OpenSecretDeveloperContext);
  if (!context) {
    throw new Error("useOpenSecretDeveloper must be used within an OpenSecretDeveloper provider");
  }
  return context;
}
