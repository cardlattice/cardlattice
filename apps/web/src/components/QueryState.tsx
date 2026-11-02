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
      <div role="status" aria-label="Loading catalogue" className="skeleton-group">
        <div className="skeleton" />
        <div className="skeleton" />
        <div className="skeleton" />
        <span className="sr-only">Loading catalogue…</span>
      </div>
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
