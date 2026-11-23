import { useQuery } from "@tanstack/react-query";
import App from "./App.tsx";
import { coverageQuery } from "../lib/coverage-query.ts";

export default function Dashboard() {
  const coverage = useQuery(coverageQuery);
  if (coverage.data) return <App rows={coverage.data.rows} revision={coverage.data.sourceCommit} />;
  if (coverage.isError)
    return (
      <main className="container">
        <h1>Catalogue Progress</h1>
        <p role="alert">{coverage.error.message}</p>
        <button type="button" onClick={() => void coverage.refetch()}>
          Retry loading coverage
        </button>
      </main>
    );
  return (
    <main className="container">
      <p role="status">Loading catalogue coverage…</p>
    </main>
  );
}
