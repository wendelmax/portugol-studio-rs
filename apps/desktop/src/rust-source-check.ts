export interface RustSourceCheckResult<T> {
  source: string;
  diagnostics: T[];
  error?: string;
}

export function createLatestOnlyRustSourceChecker<T>(
  check: (source: string) => Promise<T[]>,
) {
  let running = false;
  let pending: {
    source: string;
    resolve: (result: RustSourceCheckResult<T>) => void;
  } | null = null;

  const drain = async () => {
    if (running) return;
    running = true;
    try {
      while (pending) {
        const current = pending;
        pending = null;
        try {
          current.resolve({ source: current.source, diagnostics: await check(current.source) });
        } catch (error) {
          current.resolve({
            source: current.source,
            diagnostics: [],
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }
    } finally {
      running = false;
      if (pending) void drain();
    }
  };

  return (source: string): Promise<RustSourceCheckResult<T>> => new Promise((resolve) => {
    pending?.resolve({ source: pending.source, diagnostics: [] });
    pending = { source, resolve };
    void drain();
  });
}
