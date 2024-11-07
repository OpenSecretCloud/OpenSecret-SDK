import { useContext } from "react";
import { OpenSecretContext, OpenSecretContextType } from "./main";

export function useOpenSecret(): OpenSecretContextType {
  const context = useContext(OpenSecretContext);
  if (!context) {
    throw new Error("useOpenSecret must be used within an OpenSecretProvider");
  }
  return context;
}
