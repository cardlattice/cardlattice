import type { Coverage } from "../lib/coverage-model.ts";
export const repo = "https://github.com/cardlattice/cardlattice-data";
export const statusLabels = { empty: "Not Started", partial: "In Progress", done: "Done" };
export const n = (value: number) => new Intl.NumberFormat("en-GB").format(value);
const languageNames = new Intl.DisplayNames(["en"], { type: "language" });
export const languageName = (code: string) =>
  code === "und" ? "Not yet localised" : (languageNames.of(code) ?? code);
export const releaseLanguages = (rows: Coverage[]) =>
  new Set(rows.map((row) => row.language).filter((language) => language !== "und"));
export function groupSets(rows: Coverage[]) {
  const sets = new Map<string, Coverage[]>();
  for (const row of rows) {
    const releases = sets.get(row.setId) ?? [];
    releases.push(row);
    sets.set(row.setId, releases);
  }
  return [...sets.values()].map((releases) => ({ releases, status: groupStatus(releases) }));
}
function groupStatus(releases: Coverage[]): keyof typeof statusLabels {
  if (releases.every((row) => row.cards === 0)) {
    return "empty";
  }
  if (
    releases.every(
      (row) =>
        row.cards === row.finishes && row.cards === row.illustrators && row.cards === row.rarities,
    )
  ) {
    return "done";
  }
  return "partial";
}
export type Group = ReturnType<typeof groupSets>[number];
export function filterSets(
  groups: Group[],
  query: string,
  language: string,
  status: string,
  sort: string,
) {
  const term = query.trim().normalize("NFKC").toLowerCase();
  const filtered = groups.flatMap((group) => {
    const set = group.releases[0];
    if (
      ![set.name, set.setId, set.series, set.game]
        .join(" ")
        .normalize("NFKC")
        .toLowerCase()
        .includes(term)
    ) {
      return [];
    }
    const missing = status === "finishes" || status === "illustrators";
    if (status && !missing && status !== group.status) {
      return [];
    }
    const releases = group.releases.filter(
      (row) => (!language || row.language === language) && (!missing || row[status] < row.cards),
    );
    return releases.length ? [{ ...group, releases }] : [];
  });
  if (sort !== "default") {
    filtered.sort((a, b) => {
      const score = (group: Group) =>
        group.releases.reduce(
          (sum, row) =>
            sum + (sort === "cards" ? row.cards : sort === "gaps" ? row.cards - row.finishes : 0),
          0,
        );
      return score(b) - score(a) || a.releases[0].name.localeCompare(b.releases[0].name, "en");
    });
  }
  return filtered;
}
