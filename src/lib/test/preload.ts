interface StorageMock {
  [key: string]: string;
}

function storageMock(): Storage {
  const storage: StorageMock = {};

  return {
    setItem(key: string, value: string) {
      storage[key] = value || "";
    },
    getItem(key: string): string | null {
      return key in storage ? storage[key] : null;
    },
    removeItem(key: string) {
      delete storage[key];
    },
    clear() {
      Object.keys(storage).forEach((key) => delete storage[key]);
    },
    get length(): number {
      return Object.keys(storage).length;
    },
    key(i: number): string | null {
      const keys = Object.keys(storage);
      return keys[i] || null;
    },
    // Required Storage interface properties
    [Symbol.iterator](): IterableIterator<string> {
      return Object.keys(storage)[Symbol.iterator]();
    }
  };
}

global.localStorage = storageMock();
global.sessionStorage = storageMock();
// @ts-expect-error - window is not defined
global.window = global;

/**
 * Function to reset all global state between tests
 * This should be called in beforeEach hooks in test files
 * to ensure a clean state for each test
 */
export function resetTestState(): void {
  // Clear storage
  localStorage.clear();
  sessionStorage.clear();

  // Import and clear PCR history cache
  import("../pcrHistory")
    .then(({ clearPcrHistoryCache }) => {
      clearPcrHistoryCache();
    })
    .catch((err) => {
      console.error("Failed to clear PCR history cache:", err);
    });

  // Reset any other shared state here as needed
}

// Setup global state
// Since this file is loaded first in tests, we can also reset state at load time
resetTestState();
