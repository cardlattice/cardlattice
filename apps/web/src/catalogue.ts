import { queryClient } from "./query-client";
import { queryOptions, useQuery } from "@tanstack/react-query";
import { createCollection, useLiveQuery } from "@tanstack/react-db";
import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { loadSnapshot, manifestUrl, selectCatalogue, type CatalogueRecord } from "./catalogue-data";

export const snapshotOptions = queryOptions({
  queryKey: ["catalogue-snapshot", manifestUrl],
  queryFn: ({ signal }) => loadSnapshot(signal),
  staleTime: Infinity,
  retry: 1,
});
const records = createCollection(
  queryCollectionOptions({
    queryKey: ["catalogue-records", manifestUrl],
    staleTime: Infinity,
    retry: 1,
    queryClient,
    queryFn: async () => (await queryClient.ensureQueryData(snapshotOptions)).records,
    getKey: (record: CatalogueRecord) => record.id,
  }),
);
export function useSnapshot() {
  return useQuery(snapshotOptions);
}
export function useCatalogue<T>(path: string, enabled = true) {
  const snapshot = useSnapshot();
  const live = useLiveQuery({
    query: (q) => (enabled && snapshot.data ? q.from({ record: records }) : undefined),
  });
  const result = (() => {
    if (!enabled || !snapshot.data || live.isLoading) return { data: undefined, error: null };
    try {
      return { data: selectCatalogue(live.data || [], path) as T, error: null };
    } catch (error) {
      return {
        data: undefined,
        error: error instanceof Error ? error : new Error("Invalid catalogue query."),
      };
    }
  })();
  const error = snapshot.error || result.error;
  return {
    data: result.data,
    error,
    isError: !!error,
    isPending: enabled && !error && (snapshot.isPending || live.isLoading),
    refetch: async () => {
      const refreshed = await snapshot.refetch();
      if (!refreshed.error) await records.utils.refetch();
    },
  };
}
