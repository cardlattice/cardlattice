// @vitest-environment jsdom
import { userEvent } from "@testing-library/user-event";
import { hydrateRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { renderToString } from "react-dom/server";
import App from "./App.tsx";
import ReleaseLabel from "./ReleaseLabel.tsx";
import { filterSets, groupSets, languageName } from "../lib/model.ts";
import type { Coverage } from "../lib/coverage-model.ts";
const revision = "a".repeat(40);
const row = (overrides: Partial<Coverage> = {}): Coverage => ({
  key: "a",
  setId: "pokemon.a",
  name: "Alpha",
  game: "Pokémon",
  gameId: "pokemon",
  series: "Original",
  language: "en",
  region: "US",
  cards: 2,
  finishes: 1,
  illustrators: 1,
  rarities: 2,
  ...overrides,
});
const rows = [
  row(),
  row({ key: "aj", language: "ja", region: "JP" }),
  row({
    key: "b",
    setId: "pokemon.b",
    name: "Beta",
    cards: 1,
    finishes: 1,
    illustrators: 1,
    rarities: 1,
  }),
  row({
    key: "c",
    setId: "pokemon.c",
    name: "Empty",
    language: "und",
    region: "global",
    cards: 0,
    finishes: 0,
    illustrators: 0,
    rarities: 0,
  }),
];
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  localStorage.clear();
  delete document.documentElement.dataset.theme;
});
async function choose(label: string, option: string) {
  const trigger = screen.getByRole("combobox", { name: label });
  await new Promise((resolve) => requestAnimationFrame(resolve));
  trigger.focus();
  await userEvent.keyboard("{ArrowDown}");
  await userEvent.click(await screen.findByRole("option", { name: option }));
  await waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
}
describe("dashboard", () => {
  it("renders totals, source links, statuses, and a complete no-JavaScript page", () => {
    const html = renderToString(<App rows={rows} revision={revision} />);
    expect(html).toContain("Catalogue Progress");
    expect(html).toContain("inert");
    expect(html).toContain(`/blob/${revision}/catalogue/pokemon/sets/c.yaml`);
    render(<App rows={rows} revision={revision} />);
    expect(screen.getByRole("status").textContent).toBe("3 sets found");
    expect(screen.getByText("Known Printings").nextElementSibling?.textContent).toBe("5");
    expect(screen.getByText("Done", { selector: "[data-ui='badge']" })).toBeTruthy();
    expect(
      screen.getByLabelText("View Alpha · English · US source on GitHub").getAttribute("href"),
    ).toContain(`/tree/${revision}/catalogue/pokemon/cards/a/en/US`);
    expect(
      document.querySelector("[data-ui='coverage-ring'][data-complete='true']"),
    ).not.toBeNull();
  });
  it("filters, clears, sorts, and prevents form navigation", async () => {
    render(<App rows={rows} revision={revision} />);
    const query = screen.getByRole("combobox", { name: "Find a set, game, series or ID" });
    fireEvent.submit(screen.getByRole("search"));
    fireEvent.change(query, { target: { value: "Ａｌｐｈａ" } });
    expect(screen.getByRole("status").textContent).toBe("1 set found");
    await choose("Language", "Japanese");
    expect(document.querySelectorAll("tbody tr").length).toBe(1);
    await choose("Status", "Done");
    expect(screen.getByRole("status").textContent).toBe("0 sets found");
    expect(screen.getByText("No sets match these filters.").hidden).toBe(false);
    fireEvent.click(screen.getByText("Clear filters"));
    expect(document.activeElement).toBe(query);
    expect(screen.getByRole("status").textContent).toBe("3 sets found");
    await choose("Sort by", "Most printings");
    expect(document.querySelector("[data-ui='set-name']")?.textContent).toBe("Alpha");
    await choose("Status", "Missing finishes");
    expect(screen.getByRole("status").textContent).toBe("1 set found");
    await choose("Status", "Missing illustrators");
    expect(screen.getByRole("status").textContent).toBe("1 set found");
  });
  it("paginates, focuses the next page and resets page on filtering or page-size changes", async () => {
    const many = Array.from({ length: 51 }, (_, i) =>
      row({ key: String(i), setId: `pokemon.${i}`, name: `Set ${i}` }),
    );
    render(<App rows={many} revision={revision} />);
    expect(document.querySelectorAll("[data-ui='set-card']").length).toBe(25);
    fireEvent.click(screen.getByText("Next"));
    expect(screen.getByText("26–50 of 51")).toBeTruthy();
    await waitFor(() =>
      expect(document.activeElement).toBe(document.querySelector("[data-ui='set-card'] summary")),
    );
    fireEvent.click(screen.getByText("Previous"));
    expect(screen.getByText("1–25 of 51")).toBeTruthy();
    fireEvent.click(screen.getByText("Next"));
    await choose("Sets per page", "50");
    expect(screen.getByText("1–50 of 51")).toBeTruthy();
    fireEvent.click(screen.getByText("Next"));
    expect(screen.getByText("51–51 of 51")).toBeTruthy();
    expect(screen.getByText("Next").hasAttribute("disabled")).toBe(true);
    fireEvent.change(screen.getByRole("combobox", { name: "Find a set, game, series or ID" }), {
      target: { value: "Set 0" },
    });
    expect(screen.getByRole("status").textContent).toBe("1 set found");
    await new Promise((resolve) => setTimeout(resolve, 30));
  });
  it("handles an empty catalogue", () => {
    render(<App rows={[]} revision={revision} />);
    expect(screen.getByText("No sets are recorded yet.").hidden).toBe(false);
    expect(screen.getByText("0–0 of 0")).toBeTruthy();
    expect(screen.getByText("0% done")).toBeTruthy();
  });
  it("labels flags and distinguishes releases with the same language", () => {
    const au = row({ region: "AU" });
    const us = row();
    const { container } = render(<ReleaseLabel row={us} releases={[us, au]} />);
    expect(container.querySelector("[data-ui='release-region']")?.textContent).toContain(
      "United States",
    );
    cleanup();
    render(<ReleaseLabel row={au} releases={[au]} />);
    expect(screen.getByText("· Australia")).toBeTruthy();
    cleanup();
    const fr = row({ language: "eo", region: "FR" });
    render(<ReleaseLabel row={fr} releases={[fr]} />);
    expect(document.querySelector("[data-ui='release-language'] > svg")).toBeTruthy();
    expect(languageName("und")).toBe("Not yet localised");
  });
  it("sorts by printings, gaps, names and stable name tie breaks", () => {
    const groups = groupSets(rows);
    expect(filterSets(groups, "", "", "", "cards").map((g) => g.releases[0].name)).toEqual([
      "Alpha",
      "Beta",
      "Empty",
    ]);
    expect(filterSets(groups, "", "", "", "gaps").map((g) => g.releases[0].name)).toEqual([
      "Alpha",
      "Beta",
      "Empty",
    ]);
    expect(
      filterSets([...groups].reverse(), "", "", "", "name").map((g) => g.releases[0].name),
    ).toEqual(["Alpha", "Beta", "Empty"]);
    expect(filterSets(groups, "", "", "empty", "default")).toHaveLength(1);
    expect(filterSets(groups, "", "xx", "", "default")).toHaveLength(0);
  });
});

it("hydrates the server-rendered dashboard without mismatches", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  container.innerHTML = renderToString(<App rows={rows} revision={revision} />);
  const onRecoverableError = vi.fn();
  let root: ReturnType<typeof hydrateRoot>;
  await act(async () => {
    root = hydrateRoot(container, <App rows={rows} revision={revision} />, { onRecoverableError });
  });
  expect(onRecoverableError).not.toHaveBeenCalled();
  expect(container.querySelector("form")?.hasAttribute("inert")).toBe(false);
  await act(async () => root.unmount());
  container.remove();
});

it("opens and closes a set's coverage details", async () => {
  render(<App rows={rows} revision={revision} />);
  const summary = screen.getByLabelText("Alpha");
  const details = summary.closest("details");
  expect(details?.open).toBe(false);
  await userEvent.click(summary);
  await waitFor(() => expect(details?.open).toBe(true));
  await userEvent.click(summary);
  await waitFor(() => expect(details?.open).toBe(false));
});
it("uses the language code when a display name is unavailable", () => {
  vi.spyOn(Intl.DisplayNames.prototype, "of").mockReturnValue(undefined);
  expect(languageName("zz")).toBe("zz");
});
it("leaves theme selection to the system without reading saved preferences", () => {
  const read = vi.spyOn(Storage.prototype, "getItem");
  const write = vi.spyOn(Storage.prototype, "setItem");
  render(<App rows={rows} revision={revision} />);
  expect(screen.queryByRole("combobox", { name: "Theme" })).toBeNull();
  expect(read).not.toHaveBeenCalled();
  expect(write).not.toHaveBeenCalled();
});
