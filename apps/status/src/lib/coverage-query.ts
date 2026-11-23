import { queryOptions } from "@tanstack/react-query";
import type { Coverage } from "./coverage-model.ts";
import { loadCoverage } from "./load-coverage.ts";
export interface CoverageData {
  rows: Coverage[];
  sourceCommit: string;
  snapshotSha256: string;
}
export const coverageQuery = queryOptions({
  queryKey: ["catalogue-coverage"],
  queryFn: ({ signal }) => loadCoverage(signal),
  staleTime: 60_000,
  retry: 1,
});
