import type { ReactNode } from "react";
import { Button } from "./Button";

export function QueryState({
  pending,
  error,
  retry,
  children,
}: {
  pending: boolean;
  error: Error | null;
  retry?: () => unknown;
  children: ReactNode;
}) {
  if (pending)
    return (
      <output aria-label="Loading catalogue" className="skeleton-group">
        <span className="skeleton" />
        <span className="skeleton" />
        <span className="skeleton" />
        <span className="sr-only">Loading catalogue…</span>
      </output>
    );
  if (error)
    return (
      <div className="notice error" role="alert">
        <h2>We couldn’t load this data</h2>
        <p>{error.message}</p>
        {retry && <Button onClick={() => void retry()}>Try again</Button>}
      </div>
    );
  return children;
}
