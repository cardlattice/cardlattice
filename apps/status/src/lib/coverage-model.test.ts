import { describe, expect, it } from "vitest";
import { summarize, hasFinish, percent, type RecordData } from "./coverage-model.ts";
const set: RecordData = {
  id: "pokemon-test",
  metadata: { kind: "set", name: "Test", gameId: "pokemon", attributes: { printedDenominator: 1 } },
};
const printing = (
  id: string,
  language = "en",
  attributes: Record<string, unknown> = {},
): RecordData => ({
  id,
  metadata: { kind: "printing", setId: set.id, language, region: "US", attributes },
});
describe("status coverage", () => {
  it("counts physical printing records, not identities or finish options", () => {
    const rows = summarize([
      set,
      printing("one", "en", { finishes: ["holo", "normal"] }),
      printing("two", "ja"),
      { id: "identity", metadata: { kind: "card" } },
    ]);
    expect(rows.reduce((sum, r) => sum + r.cards, 0)).toBe(2);
    expect(rows.find((r) => r.language === "en")?.finishes).toBe(1);
  });
  it("counts named finishes but not edition-only variants or empty strings", () => {
    expect(hasFinish({ variants: { first: { edition: "first-edition" } } })).toBe(false);
    expect(hasFinish({ variants: { foil: { finish: "holo" } } })).toBe(true);
    expect(hasFinish({ finishes: [""] })).toBe(false);
  });
  it("does not invent a target from printed denominators or treat missing fields as complete", () => {
    const rows = summarize([set, printing("1"), printing("2")]);
    expect(rows[0].cards).toBe(2);
    expect(rows[0].finishes).toBe(0);
    expect(rows[0]).not.toHaveProperty("expected");
  });
  it("keeps empty sets visible and supports an entirely empty catalogue", () => {
    expect(summarize([])).toEqual([]);
    expect(summarize([set])[0].cards).toBe(0);
    expect(percent(0, 0)).toBe(0);
  });
  it("rejects duplicate records and dangling sets", () => {
    expect(() => summarize([set, printing("x"), printing("x")])).toThrow("Duplicate");
    expect(() => summarize([printing("x")])).toThrow("Unknown");
  });
});

it("handles incomplete legacy metadata without inventing coverage", () => {
  const rows = summarize([
    { id: "unnamed", metadata: { kind: "set" } },
    { id: "printing", metadata: { kind: "printing", setId: "unnamed" } },
  ]);
  expect(rows[0]).toMatchObject({
    name: "unnamed",
    game: "Unknown game",
    gameId: "unknown",
    series: "Unassigned series",
    language: "und",
    region: "global",
    cards: 1,
    finishes: 0,
  });
  expect(() => summarize([{ id: "missing-set", metadata: { kind: "printing" } }])).toThrow(
    "Unknown printing set",
  );
});
