import type { Coverage } from "../lib/coverage-model.ts";
import LanguageIcon from "./LanguageIcon.tsx";
import { languageName } from "../lib/model.ts";
const defaultRegions: Record<string, string> = {
  en: "US",
  ja: "JP",
  ko: "KR",
  nl: "NL",
  "zh-Hans": "CN",
};
const regionNames = new Intl.DisplayNames(["en"], { type: "region" });
export default function ReleaseLabel({ row, releases }: { row: Coverage; releases: Coverage[] }) {
  const region = row.region === "global" ? "Global" : regionNames.of(row.region);
  const showRegion =
    releases.some((other) => other !== row && other.language === row.language) ||
    (row.region !== "global" && row.region !== defaultRegions[row.language]);
  return (
    <span className="release-language">
      <LanguageIcon language={row.language} />
      <span>
        {languageName(row.language)}
        <span className={showRegion ? "release-region" : "sr-only"}> · {region}</span>
      </span>
    </span>
  );
}
