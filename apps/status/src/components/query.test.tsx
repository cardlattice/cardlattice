// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { loadCoverage } from "../lib/load-coverage.ts";
vi.mock("../lib/load-coverage.ts", () => ({ loadCoverage: vi.fn() }));
import Dashboard from "./Dashboard.tsx";
import ThemePicker from "./ThemePicker.tsx";
import { coverageQuery } from "../lib/coverage-query.ts";
const data = { rows: [], sourceCommit: "a".repeat(40), snapshotSha256: "b".repeat(64) };
const clients: QueryClient[] = [];
function client() {
  const result = new QueryClient({
    defaultOptions: { queries: { retryDelay: 0, gcTime: Infinity } },
  });
  clients.push(result);
  return result;
}
afterEach(() => {
  cleanup();
  for (const value of clients) value.clear();
  clients.length = 0;
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.mocked(loadCoverage).mockReset();
  localStorage.clear();
});
it("shows loading then data, and reuses cached coverage on remount", async () => {
  let complete!: (response: typeof data) => void;
  const fetcher = vi.fn(
    () =>
      new Promise<typeof data>((resolve) => {
        complete = resolve;
      }),
  );
  vi.mocked(loadCoverage).mockImplementation(fetcher);
  const cache = client();
  const first = render(
    <QueryClientProvider client={cache}>
      <Dashboard />
    </QueryClientProvider>,
  );
  expect(screen.getByRole("status").textContent).toBe("Loading catalogue coverage…");
  complete(data);
  await screen.findByText("No sets are recorded yet.");
  expect(cache.getQueryData(coverageQuery.queryKey)).toEqual(data);
  first.unmount();
  render(
    <QueryClientProvider client={cache}>
      <Dashboard />
    </QueryClientProvider>,
  );
  expect(screen.getByText("No sets are recorded yet.")).toBeTruthy();
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it("retries a failed request and offers recovery after failures", async () => {
  const fetcher = vi.fn().mockRejectedValue(new Error("Could not load coverage (HTTP 503)"));
  vi.mocked(loadCoverage).mockImplementation(fetcher);
  render(
    <QueryClientProvider client={client()}>
      <Dashboard />
    </QueryClientProvider>,
  );
  expect((await screen.findByRole("alert")).textContent).toBe("Could not load coverage (HTTP 503)");
  expect(fetcher).toHaveBeenCalledTimes(2);
  fetcher.mockResolvedValue(data);
  fireEvent.click(screen.getByRole("button", { name: "Retry loading coverage" }));
  await screen.findByText("No sets are recorded yet.");
  expect(screen.queryByRole("alert")).toBeNull();
});
it("cancels an in-flight request when the dashboard unmounts", async () => {
  const fetcher = vi.fn((_signal: AbortSignal) => new Promise<typeof data>(() => {}));
  vi.mocked(loadCoverage).mockImplementation(fetcher);
  const view = render(
    <QueryClientProvider client={client()}>
      <Dashboard />
    </QueryClientProvider>,
  );
  await waitFor(() => expect(fetcher).toHaveBeenCalledOnce());
  const signal = fetcher.mock.calls[0][0];
  expect(signal.aborted).toBe(false);
  view.unmount();
  expect(signal.aborted).toBe(true);
});
it("synchronizes theme changes from other tabs", () => {
  render(<ThemePicker />);
  act(() => {
    localStorage.setItem("cardlattice-theme", "dark");
    window.dispatchEvent(new Event("storage"));
  });
  expect(screen.getByRole("combobox", { name: "Theme" }).textContent).toBe("Dark");
  expect(document.documentElement.dataset.theme).toBe("dark");
});
