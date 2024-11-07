import { useEffect, useRef } from "react";

export function useOnMount(callback: () => void) {
  const hasRun = useRef(false);

  useEffect(() => {
    if (!hasRun.current) {
      hasRun.current = true;
      callback();
    }
  }, [callback]);
}
