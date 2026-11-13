import { createHash } from "node:crypto";
import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const sourceCommit = "a".repeat(40);
const bundle = JSON.stringify({
  schemaVersion: 1,
  sourceCommit,
  records: [
    { id: "pokemon", metadata: { kind: "game", name: "Pokémon", slug: "pokemon" } },
    {
      id: "pokemon-original-set",
      metadata: { kind: "set", gameId: "pokemon", name: "Original test set" },
    },
    {
      id: "original-card",
      metadata: {
        kind: "card",
        gameId: "pokemon",
        name: "Café Seed",
        language: "nl",
        attributes: { identityScope: "provisional" },
      },
    },
    {
      id: "original-printing",
      metadata: {
        kind: "printing",
        gameId: "pokemon",
        setId: "pokemon-original-set",
        cardId: "original-card",
        name: "Café Seed",
        language: "nl",
        collectorNumber: "001",
        attributes: { illustrators: ["Original Test Artist"] },
      },
    },
  ],
});
const sha256 = createHash("sha256").update(bundle).digest("hex");
const manifest = {
  schemaVersion: 1,
  sourceCommit,
  sha256,
  filename: `catalogue-${sha256}.json`,
  byteSize: Buffer.byteLength(bundle),
  counts: { game: 1, set: 1, card: 1, printing: 1 },
};

test.beforeEach(async ({ page }) => {
  await page.route("**/data/manifest.json", (route) => route.fulfill({ json: manifest }));
  await page.route("**/data/catalogue-*.json", (route) =>
    route.fulfill({ body: bundle, contentType: "application/json" }),
  );
});

test("browse and search a static snapshot without an API", async ({ page }) => {
  const apiRequests: string[] = [];
  page.on("request", (request) => {
    if (new URL(request.url()).pathname.startsWith("/v1/")) apiRequests.push(request.url());
  });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Card data.");
  await page.getByRole("link", { name: "Explore the catalogue", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(
    "A catalogue, built in the open.",
  );
  await page.getByRole("link", { name: /Pokémon/ }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Pokémon");
  await page.getByRole("link", { name: "Cards", exact: true }).click();
  await page.getByRole("link", { name: /Café Seed/ }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Café Seed");
  await page.reload();
  await expect(page.getByRole("heading", { name: "Related printings" })).toBeVisible();
  await page.getByRole("link", { name: "Browse printings →" }).click();
  await expect(page).toHaveURL(/cardId=original-card/);
  await expect(page.getByRole("link", { name: /Café Seed/ })).toHaveCount(1);
  await page.goto("/search?q=caf%C3%A9&language=nl");
  await expect(page.getByRole("link", { name: /Café Seed/ })).toHaveCount(2);
  await page.getByRole("textbox", { name: "Search catalogue" }).fill("no matching record");
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.getByRole("heading", { name: "No records here yet" })).toBeVisible();
  await page.goto("/docs");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("The API is coming soon.");
  await page.goto("/datasets");
  await expect(page.getByRole("link", { name: "Download JSON" })).toHaveAttribute(
    "href",
    new RegExp(manifest.filename),
  );
  expect(apiRequests).toEqual([]);
});

test("failed downloads show a retryable error", async ({ page }) => {
  await page.route("**/data/manifest.json", (route) =>
    route.fulfill({ status: 503, body: "Unavailable" }),
  );
  await page.goto("/games");
  await expect(page.getByRole("alert")).toContainText("503");
  await page.route("**/data/manifest.json", (route) => route.fulfill({ json: manifest }));
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByRole("link", { name: /Pokémon/ })).toBeVisible();
});

for (const theme of ["light", "dark"]) {
  test(`static pages are accessible on a phone: ${theme}`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 1000 });
    await page.addInitScript((value) => localStorage.setItem("cardlattice-theme", value), theme);
    for (const path of [
      "/",
      "/games",
      "/games/pokemon/cards/original-card",
      "/docs",
      "/datasets",
      "/contribute",
      "/moderate",
      "/coverage",
      "/status",
    ]) {
      await page.goto(path);
      await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
      await expect(page.getByLabel("Loading catalogue", { exact: true })).toHaveCount(0);
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      ).toBe(true);
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
    }
  });
}
