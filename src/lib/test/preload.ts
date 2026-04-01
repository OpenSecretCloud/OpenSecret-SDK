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

// Import setup to configure the SDK for tests
import './setup';
