import { useEffect } from "react";

// Simpler implementation compatible with React 19
export function useOnMount(callback: () => void) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    callback();
  }, []);
}

// Helper function to sleep for a specified duration
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Extend the Window interface to include our custom property
declare global {
  interface Window {
    __PLATFORM_API_URL__?: string;
  }
}
