import SelectPicker from "./SelectPicker.tsx";
// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
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
  expect(search.getAttribute("value")).toBe("Alpha");
  expect(screen.getByRole("status").textContent).toBe("1 set found");
  await user.clear(search);
  await user.type(search, "unknown");
  expect(
    await screen.findByText("No matching sets. Try a different name, game, series or ID."),
  ).toBeTruthy();
  await user.keyboard("{Escape}");
  expect(search.getAttribute("aria-expanded")).toBe("false");
  expect(search.getAttribute("value")).toBe("unknown");
  await user.tab();
  expect(search.getAttribute("value")).toBe("unknown");
});
it("opens suggestions with the trigger and selects with a pointer", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  await user.click(screen.getByRole("button", { name: "Show set suggestions" }));
  await user.click(await screen.findByRole("option", { name: "Beta" }));
  expect(screen.getByRole("status").textContent).toBe("1 set found");
  await user.click(screen.getByRole("button", { name: "Clear filters" }));
  expect(
    screen.getByRole("combobox", { name: "Find a set, game, series or ID" }).getAttribute("value"),
  ).toBe("");
  expect(screen.getByText("3 sets found")).toBeTruthy();
});
it("shows flags, selects a language, and restores all languages", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const trigger = screen.getByRole("combobox", { name: "Language" });
  await user.click(trigger);
  const japanese = await screen.findByRole("option", { name: "Japanese" });
  expect(japanese.querySelector("[data-country='jp']")?.getAttribute("data-country")).toBe("jp");
  expect(
    screen
      .getByRole("option", { name: "English" })
      .querySelector("[data-country='us']")
      ?.getAttribute("data-country"),
  ).toBe("us");
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
  expect(screen.getByText("3 sets found")).toBeTruthy();
});
it("supports language keyboard selection, Escape, and outside dismissal", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const trigger = screen.getByRole("combobox", { name: "Language" });
  trigger.focus();
  await user.keyboard("{ArrowDown}");
  await screen.findByRole("option", { name: "Japanese" });
  await user.keyboard("{End}{Enter}");
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
  expect(
    screen
      .getByRole("option", { name: "Korean" })
      .querySelector("[data-country='kr']")
      ?.getAttribute("data-country"),
  ).toBe("kr");
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
  expect(document.querySelector("[data-ui='release-language'] [data-country='nl']")).not.toBeNull();
});

it("keeps an unmatched selection labelled and lets the user recover", async () => {
  const onChange = vi.fn();
  render(
    <SelectPicker
      label="Example"
      value="removed"
      options={[{ value: "valid", label: "Available" }]}
      onChange={onChange}
    />,
  );
  const trigger = screen.getByRole("combobox", { name: "Example" });
  expect(trigger.textContent).toBe("Example");
  await userEvent.click(trigger);
  await userEvent.click(await screen.findByRole("option", { name: "Available" }));
  expect(onChange).toHaveBeenCalledWith("valid");
});

it("opens search suggestions on input click without typing and reopens after Escape", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const input = screen.getByRole("combobox", { name: "Find a set, game, series or ID" });
  await user.click(input);
  expect(await screen.findByRole("option", { name: "Alpha" })).toBeTruthy();
  expect(input.getAttribute("aria-expanded")).toBe("true");
  expect(input.getAttribute("value")).toBe("");
  await user.keyboard("{Escape}");
  expect(input.getAttribute("aria-expanded")).toBe("false");
  await user.click(input);
  await user.click(await screen.findByRole("option", { name: "Beta" }));
  expect(input.getAttribute("value")).toBe("Beta");
  expect(input.getAttribute("aria-expanded")).toBe("false");
});
it("opens search suggestions when tabbing into the input and keeps the arrow toggle usable", async () => {
  const user = userEvent.setup();
  render(<SetBrowser groups={groups} revision="abc" />);
  const input = screen.getByRole("combobox", { name: "Find a set, game, series or ID" });
  await user.tab();
  expect(document.activeElement).toBe(input);
  expect(await screen.findByRole("option", { name: "Alpha" })).toBeTruthy();
  const toggle = screen.getByLabelText("Show set suggestions");
  await user.click(toggle);
  expect(input.getAttribute("aria-expanded")).toBe("false");
  await user.click(toggle);
  await waitFor(() => expect(input.getAttribute("aria-expanded")).toBe("true"));
});
