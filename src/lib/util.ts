import { useEffect } from "react";

// Simpler implementation compatible with React 19
export function useOnMount(callback: () => void) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    callback();
  }, []);
}
