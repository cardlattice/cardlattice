import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import type { Coverage } from "../lib/coverage-model.ts";
import LanguageIcon from "./LanguageIcon.tsx";
import { languageName } from "../lib/model.ts";
const defaultRegions = new Map([
  ["en", "US"],
  ["ja", "JP"],
  ["ko", "KR"],
  ["nl", "NL"],
  ["zh-Hans", "CN"],
]);
const regionNames = new Intl.DisplayNames(["en"], { type: "region" });
export default function ReleaseLabel({ row, releases }: { row: Coverage; releases: Coverage[] }) {
  const region = row.region === "global" ? "Global" : regionNames.of(row.region);
  const showRegion =
    releases.some((other) => other !== row && other.language === row.language) ||
    (row.region !== "global" && row.region !== defaultRegions.get(row.language));
  return (
    <span data-ui="release-language" {...stylex.props(styles.releaseLanguage)}>
      <LanguageIcon language={row.language} />
      <span>
        {languageName(row.language)}
        <span
          data-ui={showRegion ? "release-region" : "sr-only"}
          {...stylex.props(showRegion ? styles.releaseRegion : styles.srOnly)}
        >
          {" "}
          · {region}
        </span>
      </span>
    </span>
  );
}
