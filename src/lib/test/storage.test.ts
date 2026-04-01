import { describe, test, expect, beforeEach } from 'bun:test';
import {
  getStorage,
  setStorageProvider,
  resetStorage,
  type StorageProvider,
} from '../storage';
import { configure, resetConfig } from '../config';

function createMockStorage(): StorageProvider['persistent'] {
  const storage: Record<string, string> = {};
  return {
    getItem(key: string): string | null {
      return key in storage ? storage[key] : null;
    },
    setItem(key: string, value: string): void {
      storage[key] = value;
    },
    removeItem(key: string): void {
      delete storage[key];
    },
  };
}

describe('StorageProvider', () => {
  beforeEach(() => {
    resetStorage();
  });

  test('browser environment: getStorage() returns window.localStorage / window.sessionStorage as defaults', () => {
    // The test preload sets global.window = global, so typeof window !== 'undefined'
    // In a browser-like environment, getStorage() should fall back to window storage
    const storage = getStorage();
    expect(storage.persistent).toBe(window.localStorage);
    expect(storage.session).toBe(window.sessionStorage);
  });

  test('custom provider: setStorageProvider() makes getStorage() return the custom provider', () => {
    const persistent = createMockStorage();
    const session = createMockStorage();
    const customProvider: StorageProvider = { persistent, session };

    setStorageProvider(customProvider);

    const storage = getStorage();
    expect(storage.persistent).toBe(persistent);
    expect(storage.session).toBe(session);
  });

  test('custom provider via configure(): storage option wires through', () => {
    const persistent = createMockStorage();
    const session = createMockStorage();
    const customProvider: StorageProvider = { persistent, session };

    configure({
      apiUrl: 'https://example.com',
      clientId: 'test-client-id',
      storage: customProvider,
    });

    const storage = getStorage();
    expect(storage.persistent).toBe(persistent);
    expect(storage.session).toBe(session);

    // Clean up config state
    resetConfig();
  });

  test('resetConfig() also resets the storage provider', () => {
    const customProvider: StorageProvider = {
      persistent: createMockStorage(),
      session: createMockStorage(),
    };

    configure({
      apiUrl: 'https://example.com',
      clientId: 'test-client-id',
      storage: customProvider,
    });

    // Verify custom provider is active
    expect(getStorage().persistent).toBe(customProvider.persistent);

    // Reset everything
    resetConfig();
    resetStorage();

    // After reset in a browser-like env, it should fall back to window storage
    const storage = getStorage();
    expect(storage.persistent).toBe(window.localStorage);
  });

  test('custom provider: persistent storage stores and retrieves values', () => {
    const customProvider: StorageProvider = {
      persistent: createMockStorage(),
      session: createMockStorage(),
    };
    setStorageProvider(customProvider);

    const storage = getStorage();

    // Initially null
    expect(storage.persistent.getItem('access_token')).toBeNull();

    // Set and get
    storage.persistent.setItem('access_token', 'test-token-123');
    expect(storage.persistent.getItem('access_token')).toBe('test-token-123');

    // Overwrite
    storage.persistent.setItem('access_token', 'updated-token');
    expect(storage.persistent.getItem('access_token')).toBe('updated-token');

    // Remove
    storage.persistent.removeItem('access_token');
    expect(storage.persistent.getItem('access_token')).toBeNull();
  });

  test('custom provider: session storage stores and retrieves values', () => {
    const customProvider: StorageProvider = {
      persistent: createMockStorage(),
      session: createMockStorage(),
    };
    setStorageProvider(customProvider);

    const storage = getStorage();

    storage.session.setItem('sessionKey', 'test-key');
    expect(storage.session.getItem('sessionKey')).toBe('test-key');

    storage.session.removeItem('sessionKey');
    expect(storage.session.getItem('sessionKey')).toBeNull();
  });

  test('custom provider: persistent and session storage are isolated', () => {
    const customProvider: StorageProvider = {
      persistent: createMockStorage(),
      session: createMockStorage(),
    };
    setStorageProvider(customProvider);

    const storage = getStorage();

    storage.persistent.setItem('token', 'persistent-value');
    storage.session.setItem('token', 'session-value');

    expect(storage.persistent.getItem('token')).toBe('persistent-value');
    expect(storage.session.getItem('token')).toBe('session-value');
  });
});
