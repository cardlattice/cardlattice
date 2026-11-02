import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("browse, search, deep links, history, empty state, and API explorer", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Card data.");
  await page.getByRole("link", { name: "Explore the catalogue" }).click();
  await page.getByRole("link", { name: /Pokémon/ }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Pokémon");
  await page.getByRole("link", { name: "Cards", exact: true }).click();
  await page.getByRole("link", { name: /Prism Seed — synthetic/ }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Prism Seed — synthetic");
  await page.reload();
  await expect(page.getByRole("heading", { name: "Sources & provenance" })).toBeVisible();
  await page.goBack();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Card identities.");
  await page.goto("/search?q=caf%C3%A9");
  await expect(page.getByRole("link", { name: /Café Lantern/ })).toHaveCount(2);
  await page
    .getByRole("textbox", { name: "Search catalogue" })
    .fill("no-matching-original-fixture");
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.getByRole("heading", { name: "No records here yet" })).toBeVisible();
  await page.goto("/docs");
  await page.getByRole("button", { name: "Send GET request" }).click();
  await expect(page.locator(".response-panel")).toContainText("HTTP 200");
  await expect(page.locator(".response-panel")).toContainText("Pokémon");
  await page.goto("/not-a-page");
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
});

test("API explorer fits a phone before and after a request", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 1000 });
  await page.goto("/docs");
  await expect(page.getByRole("button", { name: "Send GET request" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.getByRole("button", { name: "Send GET request" }).click();
  await expect(page.locator(".response-panel")).toContainText("HTTP 200");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
});

test("contribute, review, publish and invalidate through the UI", async ({ page }) => {
  const contributor = process.env.CONTRIBUTOR_TOKEN;
  const moderator = process.env.MODERATOR_TOKEN;
  expect(contributor, "CONTRIBUTOR_TOKEN is required for end-to-end tests").toBeTruthy();
  expect(moderator, "MODERATOR_TOKEN is required for end-to-end tests").toBeTruthy();
  const name = `Browser test ${Date.now()} — synthetic`;
  await page.goto("/contribute");
  await page.getByLabel("Record type").selectOption("set");
  await expect(page.getByLabel("Series (optional)")).toBeVisible();
  await page.getByLabel("Record type").selectOption("printing");
  await expect(page.getByLabel("Set", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Card identity", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Collector number")).toBeVisible();
  await page.getByLabel("Record type").selectOption("card");
  await page.getByLabel("Contributor token", { exact: true }).fill(contributor!);
  await page.getByLabel("Record name").fill(name);
  await page
    .getByLabel("Source description")
    .fill(
      "Original synthetic browser test fixture created independently for Cardlattice verification.",
    );
  await page.getByLabel("This is a synthetic development record").check();
  await page.getByLabel("I independently sourced this metadata").check();
  await page.getByRole("button", { name: "Submit for review" }).click();
  await expect(page.getByRole("heading", { name: "Your contribution is in." })).toBeVisible();
  await page.getByLabel("Front scan (optional)").setInputFiles({
    name: "original-synthetic-pixel.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      await page.evaluate(() => {
        const canvas = document.createElement("canvas");
        canvas.width = 2;
        canvas.height = 2;
        return canvas.toDataURL("image/png").split(",")[1];
      }),
      "base64",
    ),
  });
  await expect(
    page.getByRole("status").filter({ hasText: "Front scan stored privately" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Submit another record" }).click();
  await expect(page.getByLabel("Contributor token", { exact: true })).toHaveValue("");
  await expect(page.getByLabel("Record name")).toHaveValue("");
  await page.goto("/moderate");
  await page.getByLabel("Moderator token", { exact: true }).fill(moderator!);
  await page.getByRole("button", { name: "Load review queue" }).click();
  const card = page
    .locator(".review-card")
    .filter({ has: page.getByRole("heading", { name, exact: true }) });
  await card
    .getByLabel("Review reason")
    .fill("Verified original synthetic browser fixture; no third-party source or artwork.");
  await card.getByRole("button", { name: "Inspect private scan attachments" }).click();
  await expect(card.getByRole("button", { name: "Download front" })).toBeVisible();
  await card.getByRole("button", { name: "Approve metadata" }).click();
  await expect(card).toHaveCount(0);
  await page.goto(`/search?q=${encodeURIComponent(name.toLowerCase())}`);
  await expect(page.getByRole("link", { name: new RegExp(name) })).toBeVisible();
  await expect
    .poll(
      async () => {
        const response = await page.request.get("/v1/status");
        const status = await response.json();
        return status.pendingInvalidations;
      },
      { timeout: 30_000 },
    )
    .toBe(0);
  await page.goto("/datasets");
  await expect(page.getByRole("heading", { name: "No snapshots published yet" })).toBeVisible();
});

for (const theme of ["light", "dark"] as const) {
  for (const [name, width] of [
    ["phone", 390],
    ["tablet", 768],
    ["desktop", 1440],
  ] as const) {
    test(`accessibility and layout: ${theme}, ${name}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 1000 });
      await page.emulateMedia({ colorScheme: theme, reducedMotion: "reduce" });
      await page.goto("/");
      await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
      await expect(page.locator(".metrics-strip")).not.toContainText("—");
      const result = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
        .analyze();
      expect(result.violations).toEqual([]);
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      ).toBe(true);
      await page.screenshot({
        path: testInfo.outputPath(`home-${theme}-${name}.png`),
        fullPage: true,
      });
      await page.keyboard.press("Tab");
      await expect(page.getByRole("link", { name: "Skip to content" })).toBeFocused();
      await page.keyboard.press("Enter");
      await expect(page.locator("main")).toBeFocused();
    });
  }
}

test("forms and resource screens meet automated accessibility checks", async ({ page }) => {
  for (const route of [
    "/contribute",
    "/moderate",
    "/docs",
    "/coverage",
    "/status",
    "/games/pokemon/cards",
    "/datasets",
    "/licensing",
  ]) {
    await page.goto(route);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.locator(".skeleton-group")).toHaveCount(0);
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
      .analyze();
    expect(result.violations, route).toEqual([]);
  }
});

test("detail links keep printing collections scoped to their parent", async ({ page }) => {
  const setId = "01900000-0000-7000-8000-000000000003";
  const cardId = "01900000-0000-7000-8000-000000000004";
  for (const [collection, id, filter] of [
    ["sets", setId, "setId"],
    ["cards", cardId, "cardId"],
  ]) {
    await page.goto(`/games/pokemon/${collection}/${id}`);
    await page.getByRole("link", { name: "Browse printings →" }).click();
    await expect(page).toHaveURL(new RegExp(`${filter}=${id}`));
    await expect(page.getByRole("link", { name: /Prism Seed/ }).first()).toBeVisible();
    await page.getByRole("combobox", { name: "Language", exact: true }).selectOption("en");
    await expect(page).toHaveURL(new RegExp(`${filter}=${id}`));
    await page.reload();
    await expect(page.getByRole("link", { name: /Prism Seed/ }).first()).toBeVisible();
  }
});

test("long provisional identity metadata fits a phone", async ({ page }) => {
  const path = "/v1/games/pokemon/cards/01900000-0000-7000-8000-000000000004";
  await page.route(`**${path}`, async (route) => {
    const response = await route.fetch();
    const body = await response.json();
    body.data.attributes.identityScope = "01900000-0000-7000-8000-000000000005";
    await route.fulfill({ response, json: body });
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(path.replace("/v1", ""));
  await expect(page.getByText(/This card identity is provisional/)).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(result.violations).toEqual([]);
});
