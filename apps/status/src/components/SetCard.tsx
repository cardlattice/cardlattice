import { useState } from "react";
import { statuses } from "../lib/status.ts";
import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import ExternalIcon from "./icons/ExternalIcon.tsx";
import ChevronIcon from "./icons/ChevronIcon.tsx";
import GlobeIcon from "./icons/GlobeIcon.tsx";
import { languageName, n, releaseLanguages, repo, statusLabels, type Group } from "../lib/model.ts";

import ReleaseLabel from "./ReleaseLabel.tsx";
import Recorded from "./Recorded.tsx";
export default function SetCard({ group, revision }: { group: Group; revision: string }) {
  const [open, setOpen] = useState(false);
  const { releases, status } = group;
  const set = releases[0];
  const count = releaseLanguages(releases).size;
  const folder = `catalogue/${encodeURIComponent(set.gameId)}`;
  const slug = encodeURIComponent(set.setId.slice(set.gameId.length + 1));
  return (
    <details
      onToggle={(event) => setOpen(event.currentTarget.open)}
      data-ui="set-card"
      {...stylex.props(styles.card)}
    >
      <summary {...stylex.props(styles.summary)} aria-label={set.name}>
        <span data-ui="set-title" {...stylex.props(styles.setTitle)}>
          <span data-ui="set-name" {...stylex.props(styles.setName)}>
            {set.name}
          </span>
          <span data-ui="set-id secondary" {...stylex.props(styles.setId, styles.secondary)}>
            {set.setId}
          </span>
          <span
            data-ui="set-languages secondary"
            {...stylex.props(styles.setLanguages, styles.secondary)}
          >
            <GlobeIcon />
            {n(count)} {count === 1 ? "Language" : "Languages"}
          </span>
        </span>
        <span data-ui="set-status" {...stylex.props(styles.setStatus)}>
          <span data-ui="badge" {...stylex.props(styles.badge, statuses[status].badge)}>
            {statusLabels[status]}
          </span>
          <span data-ui="set-chevron" {...stylex.props(styles.chevron, open && styles.chevronOpen)}>
            <ChevronIcon />
          </span>
        </span>
      </summary>
      <div data-ui="release-panel" {...stylex.props(styles.releasePanel)}>
        <div
          data-ui="release-table"
          {...stylex.props(styles.tableRegion)}
          role="region"
          aria-label={`${set.name} coverage table`}
          // The coverage table scrolls horizontally and must be keyboard-accessible.
          // oxlint-disable-next-line jsx-a11y/no-noninteractive-tabindex
          tabIndex={0}
        >
          <table {...stylex.props(styles.table)}>
            <caption data-ui="sr-only" {...stylex.props(styles.srOnly)}>
              Recorded metadata for {set.name} by language and region
            </caption>
            <colgroup>
              <col {...stylex.props(styles.languageCol)} />
              <col {...stylex.props(styles.printingsCol)} />
              <col {...stylex.props(styles.finishesCol)} />
              <col {...stylex.props(styles.illustratorCol)} />
              <col />
              <col {...stylex.props(styles.linkCol)} />
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
                  <th
                    className={stylex.props(styles.cell, styles.tableHeading).className}
                    key={label}
                    scope="col"
                  >
                    {label}
                  </th>
                ))}
                <th {...stylex.props(styles.cell, styles.tableHeading)} scope="col">
                  <span data-ui="sr-only" {...stylex.props(styles.srOnly)}>
                    Source
                  </span>
                </th>
              </tr>
            </thead>
            <tbody>
              {releases.map((row, index) => {
                const cell = stylex.props(styles.cell, index === 0 && styles.firstRowCell);
                const source = row.cards
                  ? `${repo}/tree/${revision}/${folder}/cards/${slug}/${encodeURIComponent(row.language)}/${encodeURIComponent(row.region)}`
                  : `${repo}/blob/${revision}/${folder}/sets/${slug}.yaml`;
                return (
                  <tr className={stylex.props(styles.row).className} key={row.key}>
                    <th {...cell} scope="row">
                      <ReleaseLabel row={row} releases={releases} />
                    </th>
                    <td
                      data-ui="printings"
                      {...stylex.props(
                        styles.cell,
                        index === 0 && styles.firstRowCell,
                        styles.printings,
                      )}
                    >
                      {n(row.cards)}
                    </td>
                    <td {...cell}>
                      <Recorded value={row.finishes} cards={row.cards} />
                    </td>
                    <td {...cell}>
                      <Recorded value={row.illustrators} cards={row.cards} />
                    </td>
                    <td {...cell}>
                      <Recorded value={row.rarities} cards={row.cards} ring={false} />
                    </td>
                    <td
                      data-ui="source"
                      {...stylex.props(
                        styles.cell,
                        index === 0 && styles.firstRowCell,
                        styles.sourceCell,
                      )}
                    >
                      <a
                        {...stylex.props(styles.sourceLink)}
                        href={source}
                        aria-label={`View ${set.name} · ${languageName(row.language)} · ${row.region} source on GitHub`}
                      >
                        <ExternalIcon />
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
