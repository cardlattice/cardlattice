import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
export default function Footer() {
  return (
    <footer data-ui="container page-footer" {...stylex.props(styles.container, styles.footer)}>
      <details data-ui="method" {...stylex.props(styles.method)}>
        <summary {...stylex.props(styles.methodSummary)}>How progress is measured</summary>
        <p>
          Progress is the share of sets at each status. A set is not started if it has no printings,
          in progress if any known printing lacks a finish, illustrator or rarity, and done when
          every known printing has all three recorded.
        </p>
        <p>
          Counts cover known printings in each language and region. A printing counts once even if
          it has multiple finish options.
        </p>
      </details>
      <div data-ui="footer-links" {...stylex.props(styles.footerLinks)}>
        <a href="https://cardlattice.com">Browse catalogue</a>
        <a href="https://cardlattice-status.pages.dev/data/manifest.json">Catalogue manifest</a>
      </div>
    </footer>
  );
}

const styles = stylex.create({
  container: {
    width: {
      default: "min(870px, calc(100% - 32px))",
      "@media (max-width: 390px)": "calc(100% - 24px)",
    },
    marginInline: "auto",
  },
  footer: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "start",
    flexWrap: "wrap",
    gap: 16,
    marginTop: 48,
    paddingTop: 20,
    paddingBottom: 34,
    borderTopWidth: 1,
    borderTopStyle: "solid",
    borderTopColor: colors.border,
    color: colors.secondary,
    fontSize: 13,
  },
  method: { maxWidth: 460 },
  methodSummary: { minHeight: 32, alignContent: "center" },
  footerLinks: {
    display: "flex",
    flexWrap: "wrap",
    alignItems: "center",
    rowGap: 8,
    columnGap: 16,
    fontSize: 13,
    color: colors.secondary,
  },
});
