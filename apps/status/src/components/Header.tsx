import { github, wordmark } from "../lib/artwork.ts";
import { repo } from "../lib/model.ts";
export default function Header({ revision }: { revision: string }) {
  return (
    <header className="container page-header">
      <a
        className="wordmark"
        href="https://cardlattice.com"
        target="_blank"
        rel="noopener"
        aria-label="Cardlattice home"
        dangerouslySetInnerHTML={{ __html: wordmark }}
      />
      <nav className="header-actions" aria-label="Project">
        <div className="header-buttons">
          <a
            className="header-button commit-button"
            href={`${repo}/commit/${revision}`}
            target="_blank"
            rel="noopener"
          >
            <span className="commit-dot" aria-hidden="true" />
            Last commit: {revision.slice(0, 7)}
          </a>
          <a
            className="header-button primary-button"
            href={`${repo}/blob/main/CONTRIBUTING.md`}
            target="_blank"
            rel="noopener"
          >
            Contribution Guide
          </a>
        </div>
        <a
          className="github-link"
          href={repo}
          target="_blank"
          rel="noopener"
          aria-label="Cardlattice data on GitHub"
          dangerouslySetInnerHTML={{ __html: github }}
        />
      </nav>
    </header>
  );
}
