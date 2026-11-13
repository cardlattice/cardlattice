import { useQuery } from "@tanstack/react-query";

import type { CatalogueRecord, Page, Stats, RecordDetail } from "./generated/catalogue";

export type RecordData = Omit<
  CatalogueRecord,
  "attributes" | "createdAt" | "updatedAt" | "synthetic"
> & {
  attributes: Record<string, unknown>;
  createdAt?: string;
  updatedAt?: string;
  synthetic?: boolean;
};
export type PageData = Omit<Page, "data"> & { data: RecordData[] };
export type StatsData = Stats;
export type DetailData = Omit<RecordDetail, "data" | "printings"> & {
  data: RecordData;
  printings: RecordData[];
};
export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}
export async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, options);
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new ApiError(
      response.status,
      body?.error?.message ?? `Request failed (${response.status})`,
    );
  }
  return response.json() as Promise<T>;
}
export function useApi<T>(path: string, enabled = true) {
  return useQuery({
    queryKey: [path],
    queryFn: ({ signal }) => request<T>(path, { signal }),
    enabled,
    staleTime: 30_000,
    retry: (count, error) => count < 1 && (!(error instanceof ApiError) || error.status >= 500),
  });
}
export function authorized(token: string, body?: unknown): RequestInit {
  return {
    method: body === undefined ? "GET" : "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      ...(body === undefined ? {} : { "Content-Type": "application/json" }),
    },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  };
}
export const number = new Intl.NumberFormat("en");
const dateFormatter = new Intl.DateTimeFormat("en", { dateStyle: "medium", timeStyle: "short" });
export function date(value: string) {
  return dateFormatter.format(new Date(value));
}
