// @vitest-environment jsdom
import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import SetBrowser from "./SetBrowser.tsx";
import LanguagePicker from "./LanguagePicker.tsx";
import { groupSets } from "../lib/model.ts";
import type { Coverage } from "../lib/coverage-model.ts";
const rows: Coverage[] = ["Alpha", "Beta", "Gamma"].map((name, i) => ({
  key: name,
  name,
  setId: `pokemon.${name}`,
  gameId: "pokemon",
  game: "Pokémon",
  series: "Original",
  language: i === 1 ? "ja" : "en",
  region: i === 1 ? "JP" : "US",
  cards: 1,
  finishes: 1,
  illustrators: 1,
  rarities: 1,
}));
const groups = groupSets(rows);
afterEach(cleanup);
it("selects search suggestions with the keyboard and preserves free text", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const search = screen.getByRole("combobox", { name: "Find a set, game, series or ID" });
  await user.click(search);
  await user.type(search, "Orig");
  expect(await screen.findByRole("option", { name: "Alpha" })).toBeTruthy();
  await user.keyboard("{ArrowDown}{Enter}");
  expect((search as HTMLInputElement).value).toBe("Alpha");
  expect(screen.getByRole("status").textContent).toBe("1 set found");
  await user.clear(search);
  await user.type(search, "unknown");
  expect(
    await screen.findByText("No matching sets. Try a different name, game, series or ID."),
  ).toBeTruthy();
  await user.keyboard("{Escape}");
  expect(search.getAttribute("aria-expanded")).toBe("false");
  expect((search as HTMLInputElement).value).toBe("unknown");
  await user.tab();
  expect((search as HTMLInputElement).value).toBe("unknown");
});
it("opens suggestions with the trigger and selects with a pointer", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  await user.click(screen.getByRole("button", { name: "Show set suggestions" }));
  await user.click(await screen.findByRole("option", { name: "Beta" }));
  expect(screen.getByRole("status").textContent).toBe("1 set found");
  await user.click(screen.getByRole("button", { name: "Clear filters" }));
  expect(
    (screen.getByRole("combobox", { name: "Find a set, game, series or ID" }) as HTMLInputElement)
      .value,
  ).toBe("");
  expect(screen.getByRole("status").textContent).toBe("3 sets found");
});
it("shows flags, selects a language, and restores all languages", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const trigger = screen.getByRole("combobox", { name: "Language" });
  await user.click(trigger);
  const japanese = await screen.findByRole("option", { name: "Japanese" });
  expect(japanese.querySelector(".fi-jp")?.className).toBe("fi fi-jp flag");
  expect(screen.getByRole("option", { name: "English" }).querySelector(".fi-us")?.className).toBe(
    "fi fi-us flag",
  );
  await user.click(japanese);
  expect(trigger.textContent).toBe("Japanese");
  expect(screen.getByRole("status").textContent).toBe("1 set found");
  trigger.focus();
  await user.keyboard("{ArrowDown}");
  expect(
    (await screen.findByRole("option", { name: "Japanese" })).getAttribute("aria-selected"),
  ).toBe("true");
  await user.click(screen.getByRole("option", { name: "All languages" }));
  expect(trigger.textContent).toBe("Language");
  expect(screen.getByRole("status").textContent).toBe("3 sets found");
});
it("supports language keyboard selection, Escape, and outside dismissal", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const trigger = screen.getByRole("combobox", { name: "Language" });
  trigger.focus();
  await user.keyboard("{ArrowDown}{End}{Enter}");
  expect(trigger.textContent).toBe("Japanese");
  await user.click(trigger);
  await user.keyboard("{Escape}");
  expect(trigger.getAttribute("aria-expanded")).toBe("false");
  expect(document.activeElement).toBe(trigger);
  await user.click(trigger);
  await user.click(screen.getByRole("heading", { name: "Set coverage" }));
  await waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
});
it("uses a globe fallback for unlocalised and unmapped languages", async () => {
  const user = userEvent.setup();
  render(
    <LanguagePicker
      languages={["und", "eo", "ko", "nl", "zh-Hans"]}
      value=""
      onChange={() => {}}
    />,
  );
  await user.click(screen.getByRole("combobox", { name: "Language" }));
  for (const name of ["Not yet localised", "Esperanto"]) {
    const option = await screen.findByRole("option", { name });
    expect(option.querySelector("img")).toBeNull();
    expect(option.querySelector("svg")).not.toBeNull();
  }
  expect(screen.getByRole("option", { name: "Korean" }).querySelector(".fi-kr")?.className).toBe(
    "fi fi-kr flag",
  );
});
it("bounds suggestions and resets pagination when selecting a language", async () => {
  const user = userEvent.setup();
  const many = groupSets(
    Array.from({ length: 30 }, (_, i) => ({
      ...rows[0],
      key: String(i),
      setId: `pokemon.${i}`,
      name: `Set ${i}`,
    })),
  );
  render(<SetBrowser groups={many} revision="abc" />);
  await user.click(screen.getByText("Next"));
  await user.click(screen.getByRole("combobox", { name: "Language" }));
  await user.click(await screen.findByRole("option", { name: "English" }));
  expect(screen.getByText("1–25 of 30")).toBeTruthy();
  await user.click(screen.getByRole("button", { name: "Show set suggestions" }));
  const list = await screen.findByRole("listbox", { name: "Set suggestions" });
  expect(within(list).getAllByRole("option")).toHaveLength(10);
});

it("shows the Dutch flag even for a global release", () => {
  render(
    <SetBrowser
      groups={groupSets([{ ...rows[0], language: "nl", region: "global" }])}
      revision="abc"
    />,
  );
  expect(document.querySelector(".release-language .fi-nl")).not.toBeNull();
});
