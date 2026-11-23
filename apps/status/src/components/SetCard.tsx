import { colors } from "../palette.stylex.ts";
import { useState } from "react";
import * as stylex from "@stylexjs/stylex";
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
          <span data-ui="badge" {...stylex.props(styles.badge, badgeStyles[status])}>
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

const styles = stylex.create({
  card: {
    minWidth: 0,
    borderWidth: 1,
    borderStyle: "solid",
    borderColor: colors.border,
    borderRadius: 12,
  },
  summary: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    paddingTop: 18,
    paddingRight: 19,
    paddingBottom: 16,
    paddingLeft: 15,
    borderRadius: 11,
    listStyleType: "none",
    "::-webkit-details-marker": { display: "none" },
  },
  setTitle: { display: "grid", gap: 2, minWidth: 0, overflowWrap: "anywhere" },
  setName: { lineHeight: "20px" },
  setId: { fontSize: 13, lineHeight: "18px", color: colors.secondary },
  secondary: { color: colors.secondary },
  setLanguages: {
    display: "flex",
    alignItems: "center",
    gap: 6,
    marginTop: 12,
    fontSize: 13,
    lineHeight: "20px",
    color: colors.secondary,
  },
  setStatus: {
    display: "flex",
    alignItems: "center",
    gap: { default: 18, "@media (max-width: 560px)": 6 },
  },
  badge: {
    display: "inline-flex",
    alignItems: "center",
    gap: 6,
    whiteSpace: "nowrap",
    paddingBlock: 3,
    paddingInline: 8,
    borderRadius: 8,
    fontSize: 12,
    lineHeight: "18px",
    userSelect: "none",
    "::before": {
      content: '""',
      borderRadius: "50%",
      width: 6,
      height: 6,
      backgroundColor: "currentColor",
    },
  },
  chevron: {
    color: colors.secondary,
    transform: "rotate(-90deg)",
    transitionProperty: {
      default: "none",
      "@media (prefers-reduced-motion: no-preference)": "transform",
    },
    transitionDuration: "220ms",
  },
  chevronOpen: { transform: "none" },
  releasePanel: { minWidth: 0, minHeight: 0 },
  tableRegion: {
    position: "relative",
    overflowX: "auto",
    marginTop: 0,
    marginRight: 17,
    marginBottom: 19,
    marginLeft: 15,
    borderTopLeftRadius: 12,
    borderTopRightRadius: 12,
    outlineOffset: -2,
  },
  table: {
    width: "100%",
    minWidth: 614,
    borderCollapse: "separate",
    borderSpacing: 0,
    tableLayout: "fixed",
    fontSize: 13,
    fontVariantNumeric: "tabular-nums",
  },
  srOnly: {
    position: "absolute",
    width: 1,
    height: 1,
    overflow: "hidden",
    whiteSpace: "nowrap",
    clipPath: "inset(50%)",
  },
  languageCol: { width: "21%" },
  printingsCol: { width: "24.4%" },
  finishesCol: { width: "16.25%" },
  illustratorCol: { width: "17.1%" },
  linkCol: { width: "7%" },
  cell: {
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 15,
    textAlign: "left",
    fontWeight: 400,
    borderTopWidth: 1,
    borderTopStyle: "solid",
    borderTopColor: colors.border,
  },
  tableHeading: {
    height: 32,
    borderBlockWidth: 1,
    borderBlockStyle: "solid",
    borderBlockColor: colors.border,
    color: colors.secondary,
    fontSize: 12,
    ":first-child": {
      borderLeftWidth: 1,
      borderLeftStyle: "solid",
      borderLeftColor: colors.border,
      borderTopLeftRadius: 12,
      borderBottomLeftRadius: 12,
    },
    ":last-child": {
      borderRightWidth: 1,
      borderRightStyle: "solid",
      borderRightColor: colors.border,
      borderTopRightRadius: 12,
      borderBottomRightRadius: 12,
    },
  },
  firstRowCell: { borderTopWidth: 0 },
  row: { height: 48 },
  printings: { paddingLeft: 29 },
  sourceCell: { padding: 0, textAlign: "center" },
  sourceLink: {
    display: "inline-grid",
    width: 28,
    height: 28,
    placeItems: "center",
    borderRadius: 5,
    color: { default: colors.secondary, ":hover": colors.primary },
    backgroundColor: { default: "transparent", ":hover": colors.hover },
  },
  redBadge: { backgroundColor: colors.redBadge, color: colors.redText },
  orangeBadge: { backgroundColor: colors.orangeBadge, color: colors.orangeText },
  greenBadge: { backgroundColor: colors.greenBadge, color: colors.greenText },
});

const badgeStyles = {
  empty: styles.redBadge,
  partial: styles.orangeBadge,
  done: styles.greenBadge,
};
