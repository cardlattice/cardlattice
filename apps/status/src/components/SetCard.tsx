import { languageName, n, releaseLanguages, repo, statusLabels, type Group } from "../lib/model.ts";
import Icon from "./Icon.tsx";
import ReleaseLabel from "./ReleaseLabel.tsx";
import Recorded from "./Recorded.tsx";
export default function SetCard({ group, revision }: { group: Group; revision: string }) {
  const { releases, status } = group;
  const set = releases[0];
  const count = releaseLanguages(releases).size;
  const folder = `catalogue/${encodeURIComponent(set.gameId)}`;
  const slug = encodeURIComponent(set.setId.slice(set.gameId.length + 1));
  return (
    <details className="set-card">
      <summary aria-label={set.name}>
        <span className="set-title">
          <span className="set-name">{set.name}</span>
          <span className="set-id secondary">{set.setId}</span>
          <span className="set-languages secondary">
            <Icon name="globe" />
            {n(count)} {count === 1 ? "Language" : "Languages"}
          </span>
        </span>
        <span className="set-status">
          <span className={`badge ${status}`}>{statusLabels[status]}</span>
          <span className="set-chevron">
            <Icon name="chevron" />
          </span>
        </span>
      </summary>
      <div className="release-panel">
        <div
          className="release-table"
          role="region"
          aria-label={`${set.name} coverage table`}
          // The coverage table scrolls horizontally and must be keyboard-accessible.
          // oxlint-disable-next-line jsx-a11y/no-noninteractive-tabindex
          tabIndex={0}
        >
          <table>
            <caption className="sr-only">
              Recorded metadata for {set.name} by language and region
            </caption>
            <colgroup>
              {["language", "printings", "finishes", "illustrator", "rarity", "link"].map(
                (name) => (
                  <col key={name} className={`${name}-col`} />
                ),
              )}
            </colgroup>
            <thead>
              <tr>
                {[
                  "Language",
                  "Printings",
                  "Finishes recorded",
                  "Illustrator recorded",
                  "Rarity recorded",
                ].map((label) => (
                  <th key={label} scope="col">
                    {label}
                  </th>
                ))}
                <th scope="col">
                  <span className="sr-only">Source</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {releases.map((row) => {
                const source = row.cards
                  ? `${repo}/tree/${revision}/${folder}/cards/${slug}/${encodeURIComponent(row.language)}/${encodeURIComponent(row.region)}`
                  : `${repo}/blob/${revision}/${folder}/sets/${slug}.yaml`;
                return (
                  <tr key={row.key}>
                    <th scope="row">
                      <ReleaseLabel row={row} releases={releases} />
                    </th>
                    <td className="printings">{n(row.cards)}</td>
                    <td>
                      <Recorded value={row.finishes} cards={row.cards} />
                    </td>
                    <td>
                      <Recorded value={row.illustrators} cards={row.cards} />
                    </td>
                    <td>
                      <Recorded value={row.rarities} cards={row.cards} ring={false} />
                    </td>
                    <td className="source">
                      <a
                        href={source}
                        aria-label={`View ${set.name} · ${languageName(row.language)} · ${row.region} source on GitHub`}
                      >
                        <Icon name="external" />
                      </a>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </details>
  );
}
