import { setStorageProvider, type StorageProvider } from '../storage';

function createMockStorage(): StorageProvider['persistent'] {
  const storage: Record<string, string> = {};

  return {
    getItem(key: string): string | null {
      return key in storage ? storage[key] : null;
    },
    setItem(key: string, value: string): void {
      storage[key] = value || '';
    },
    removeItem(key: string): void {
      delete storage[key];
    },
  };
}

// Configure the SDK storage provider before any other imports
setStorageProvider({
  persistent: createMockStorage(),
  session: createMockStorage(),
});

// Still needed for other browser APIs that tests may reference
// @ts-expect-error - window is not defined
global.window = global;

// Configure SDK for integration tests (skipped gracefully when env vars are missing)
try {
  await import('./setup');
} catch {
  // setup.ts throws when VITE_OPEN_SECRET_API_URL is not set;
  // that's expected for unit tests that don't need a running server.
}
