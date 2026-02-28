import { useContext } from "react";
import { OpenSecretContext, OpenSecretContextType } from "./main";

/**
 * Hook to access OpenSecret context within the provider
 * 
 * @deprecated The useOpenSecret hook is deprecated along with OpenSecretProvider. 
 * Instead, import API functions directly and use them without a provider.
 * 
 * Migration guide:
 * ```tsx
 * // Old approach (deprecated)
 * const os = useOpenSecret();
 * await os.signIn(email, password);
 * 
 * // New approach
 * import { signIn } from '@opensecret/react';
 * await signIn(email, password);
 * ```
 * 
 * @returns The OpenSecret context containing auth state and API methods
 * @throws {Error} If used outside of OpenSecretProvider
 */
export function useOpenSecret(): OpenSecretContextType {
  const context = useContext(OpenSecretContext);
  // React 19 compatibility: Don't check for nullish context since the default value is provided
  return context;
}
