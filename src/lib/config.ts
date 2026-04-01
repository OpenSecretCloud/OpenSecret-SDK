/**
 * Global configuration for OpenSecret SDK
 */

import type { StorageProvider } from './storage';
import { setStorageProvider, resetStorage } from './storage';

export interface OpenSecretConfig {
  apiUrl: string;
  clientId: string;
  /** Custom storage provider for non-browser environments (React Native, Node, etc.) */
  storage?: StorageProvider;
}

let config: OpenSecretConfig | null = null;

/**
 * Configure the OpenSecret SDK with your API URL and client ID.
 * This must be called before using any other SDK functions.
 *
 * @param options - Configuration options
 * @param options.apiUrl - The URL of your OpenSecret backend
 * @param options.clientId - Your project's client ID (UUID)
 *
 * @example
 * ```typescript
 * import { configure } from '@opensecret/react';
 *
 * configure({
 *   apiUrl: 'https://api.opensecret.cloud',
 *   clientId: '550e8400-e29b-41d4-a716-446655440000'
 * });
 * ```
 */
export function configure(options: OpenSecretConfig): void {
  if (!options.apiUrl || options.apiUrl.trim() === '') {
    throw new Error('OpenSecret SDK requires a non-empty apiUrl');
  }
  if (!options.clientId || options.clientId.trim() === '') {
    throw new Error('OpenSecret SDK requires a non-empty clientId');
  }

  if (options.storage) {
    setStorageProvider(options.storage);
  }

  config = {
    apiUrl: options.apiUrl.replace(/\/$/, ''), // Remove trailing slash
    clientId: options.clientId,
  };
}

/**
 * Get the current configuration
 * @throws {Error} If configure() hasn't been called yet
 */
export function getConfig(): OpenSecretConfig {
  if (!config) {
    throw new Error(
      'OpenSecret SDK not configured. Please call configure() with your apiUrl and clientId first.',
    );
  }
  return config;
}

/**
 * Check if the SDK has been configured
 */
export function isConfigured(): boolean {
  return config !== null;
}

/**
 * Reset configuration (mainly for testing)
 */
export function resetConfig(): void {
  config = null;
  resetStorage();
}
