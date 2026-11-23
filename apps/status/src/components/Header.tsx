import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
import GithubIcon from "./icons/GithubIcon.tsx";
import Wordmark from "./Wordmark.tsx";
import { repo } from "../lib/model.ts";
export default function Header({ revision }: { revision: string }) {
  return (
    <header data-ui="container page-header" {...stylex.props(styles.container, styles.header)}>
      <a
        data-ui="wordmark"
        {...stylex.props(styles.wordmark)}
        href="https://cardlattice.com"
        target="_blank"
        rel="noopener"
        aria-label="Cardlattice home"
      >
        <Wordmark />
      </a>
      <nav data-ui="header-actions" {...stylex.props(styles.headerActions)} aria-label="Project">
        <div data-ui="header-buttons" {...stylex.props(styles.headerButtons)}>
          <a
            data-ui="header-button commit-button"
            {...stylex.props(styles.headerButton, styles.commitButton)}
            href={`${repo}/commit/${revision}`}
            target="_blank"
            rel="noopener"
          >
            <span data-ui="commit-dot" {...stylex.props(styles.commitDot)} aria-hidden="true" />
            Last commit: {revision.slice(0, 7)}
          </a>
          <a
            data-ui="header-button primary-button"
            {...stylex.props(styles.headerButton, styles.primaryButton)}
            href={`${repo}/blob/main/CONTRIBUTING.md`}
            target="_blank"
            rel="noopener"
          >
            Contribution Guide
          </a>
        </div>
        <a
          data-ui="github-link"
          {...stylex.props(styles.githubLink)}
          href={repo}
          target="_blank"
          rel="noopener"
          aria-label="Cardlattice data on GitHub"
        >
          <GithubIcon />
        </a>
      </nav>
    </header>
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
  header: {
    display: "flex",
    alignItems: { default: "center", "@media (max-width: 560px)": "start" },
    justifyContent: "space-between",
    gap: 16,
    paddingTop: { default: 34, "@media (max-width: 560px)": 24 },
    marginBottom: { default: 54, "@media (max-width: 560px)": 34 },
    flexDirection: { default: "row", "@media (max-width: 560px)": "column" },
  },
  wordmark: {
    display: "inline-flex",
    alignItems: "center",
    gap: 10,
    fontSize: 20,
    fontWeight: 750,
    letterSpacing: "-0.06em",
    textDecoration: "none",
  },
  headerActions: { display: "flex", alignItems: "center", gap: 12, maxWidth: "100%" },
  headerButtons: { display: "flex", alignItems: "center", gap: 6 },
  headerButton: {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: 8,
    height: 32,
    paddingInline: 8,
    borderRadius: 8,
    fontSize: { default: 13, "@media (max-width: 390px)": 11 },
    whiteSpace: { default: "nowrap", "@media (max-width: 390px)": "normal" },
    textAlign: "center",
    textDecoration: "none",
  },
  commitButton: {
    borderWidth: 1,
    borderStyle: "solid",
    borderColor: colors.borderInteractive,
    backgroundColor: { default: "transparent", ":hover": colors.hover },
  },
  commitDot: {
    flexShrink: 0,
    width: 8,
    height: 8,
    borderRadius: "50%",
    backgroundColor: colors.accent,
  },
  primaryButton: {
    backgroundColor: { default: colors.accent, ":hover": colors.accentHover },
    color: "white",
  },
  githubLink: {
    display: "grid",
    flexShrink: 0,
    placeItems: "center",
    height: 32,
    borderRadius: 4,
    color: { default: colors.primary, ":hover": colors.secondary },
  },
});
