import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
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
