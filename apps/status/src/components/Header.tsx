import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
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
