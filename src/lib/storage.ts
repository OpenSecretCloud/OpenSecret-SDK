/**
 * Storage abstraction for OpenSecret SDK.
 *
 * In browser environments the SDK falls back to localStorage / sessionStorage
 * automatically.  Non-browser consumers (React Native, Node, tests) must call
 * `configure({ storage: ... })` before any other SDK usage.
 */

export type StorageProvider = {
  persistent: {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
  };
  session: {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
  };
};

let _provider: StorageProvider | null = null;

export function setStorageProvider(provider: StorageProvider): void {
  _provider = provider;
}

export function getStorage(): StorageProvider {
  if (!_provider) {
    if (typeof window !== 'undefined') {
      _provider = {
        persistent: window.localStorage,
        session: window.sessionStorage,
      };
    } else {
      throw new Error(
        'OpenSecret SDK: no storage provider configured. ' +
          'In non-browser environments, call configure({ storage: ... }) before using the SDK.',
      );
    }
  }
  return _provider;
}

export function resetStorage(): void {
  _provider = null;
}
