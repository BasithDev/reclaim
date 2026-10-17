// Reclaim: tiny async-data hook (data/loading/error/reload) shared by the A4 pages.
import { useCallback, useEffect, useRef, useState } from "react";

export function useAsync<T>(fn: () => Promise<T>, deps: unknown[]) {
  const [state, setState] = useState<{ data: T | null; loading: boolean; error: string | null }>({
    data: null, loading: true, error: null,
  });
  const fnRef = useRef(fn);
  fnRef.current = fn;
  const seq = useRef(0);

  const run = useCallback(() => {
    const id = ++seq.current;
    setState((s) => ({ ...s, loading: true, error: null }));
    fnRef.current().then(
      (data) => id === seq.current && setState({ data, loading: false, error: null }),
      (e) => id === seq.current && setState((s) => ({ data: s.data, loading: false, error: String(e) })),
    );
  }, []);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(run, deps);
  return { ...state, reload: run };
}
