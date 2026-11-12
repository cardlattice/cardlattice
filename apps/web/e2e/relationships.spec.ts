import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("products, collections and artwork retain independently browsable printings", async ({
  page,
  request,
}) => {
  const stamp = Date.now();
  const refs: Record<string, string> = {};
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  for (const kind of ["product", "collection", "artwork", "printing"]) {
    const name = `Relationship ${kind} ${stamp} — synthetic`;
    const draft =
      kind === "printing"
        ? {
            kind,
            name,
            gameId: "pokemon",
            synthetic: true,
            setId: "pokemon-set-lattice-laboratory-synthetic",
            cardId: "pokemon-card-prism-seed-synthetic",
            collectorNumber: `REL-${stamp}`,
            language: "en",
            region: "US",
            attributes: {
              products: [refs.product],
              collections: [refs.collection],
              artwork: refs.artwork,
            },
          }
        : {
            kind,
            name,
            gameId: "pokemon",
            synthetic: true,
            attributes:
              kind === "artwork"
                ? { illustrators: ["Synthetic artist"] }
                : { relatedSets: ["pokemon-set-lattice-laboratory-synthetic"] },
          };
    const submitted = await request.post("/v1/contributions", {
      headers: {
        authorization: `Bearer ${process.env.CONTRIBUTOR_TOKEN}`,
        "idempotency-key": `relationship-${stamp}-${kind}`,
      },
      data: { operation: "create", termsAccepted: true, draft },
    });
    expect(submitted.ok(), await submitted.text()).toBe(true);
    const contribution = await submitted.json();
    const reviewed = await request.post(`/v1/moderation/contributions/${contribution.id}/review`, {
      headers: { authorization: `Bearer ${process.env.MODERATOR_TOKEN}` },
      data: { decision: "approve", reason: "Verified original synthetic relationship test" },
    });
    expect(reviewed.ok(), await reviewed.text()).toBe(true);
    refs[kind] = (await reviewed.json()).recordId;
  }
  for (const kind of ["product", "collection", "artwork"]) {
    await page.goto(`/games/pokemon/${kind}s/${refs[kind]}`);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(
      `Relationship ${kind} ${stamp} — synthetic`,
    );
    await expect(
      page.getByRole("link", { name: new RegExp(`Relationship printing ${stamp}`) }),
    ).toBeVisible();
    await page.getByRole("link", { name: "Browse printings →", exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`${kind}Id=${refs[kind]}`));
    await expect(
      page.getByRole("link", { name: new RegExp(`Relationship printing ${stamp}`) }),
    ).toHaveCount(1);
    await page.goBack();
    await page.getByRole("link", { name: "Suggest a correction" }).click();
    await expect(page.getByLabel("Record type")).toHaveValue(kind);
  }
  await page.goto(`/games/pokemon/products/${refs.product}`);
  for (const width of [390, 820, 1440])
    for (const theme of ["light", "dark"] as const) {
      await page.setViewportSize({ width, height: 900 });
      await page.emulateMedia({ colorScheme: theme });
      expect(
        (
          await new AxeBuilder({ page })
            .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
            .analyze()
        ).violations,
      ).toEqual([]);
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      ).toBe(true);
    }
  expect(errors).toEqual([]);
});
