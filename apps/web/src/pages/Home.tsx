import { manifestUrl } from "../catalogue-data";
import { useCatalogue } from "../catalogue";
import { Link } from "@tanstack/react-router";
import {
  ArrowRight,
  ArrowUpRight,
  Braces,
  Database,
  GitPullRequest,
  Check,
  Terminal,
} from "lucide-react";
import { useState } from "react";
import { number, type PageData, type StatsData } from "../api";
import { Button } from "../components/Button";

export function Home() {
  const stats = useCatalogue<StatsData>("/stats");
  const example = useCatalogue<PageData>(
    "/games/pokemon/sets?limit=1",
    (stats.data?.games ?? 0) > 0,
  );
  const featured = useCatalogue<PageData>("/games/pokemon/sets?limit=3");
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const command = `curl "${manifestUrl}"`;
  const exampleText = example.data
    ? JSON.stringify(
        {
          data: example.data.data.map(({ kind, name, releaseDate, synthetic }) => ({
            kind,
            name,
            releaseDate,
            synthetic,
          })),
          limit: example.data.limit,
        },
        null,
        2,
      )
    : example.isError
      ? "The catalogue example is temporarily unavailable."
      : stats.data?.games === 0
        ? "The catalogue is empty. Import reviewed metadata to get started."
        : "Loading a published catalogue record…";
  async function copy() {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setCopyError(false);
    } catch {
      setCopyError(true);
    }
  }
  return (
    <div className="page home">
      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">
            <span className="small-square" /> THE OPEN CARD CATALOGUE
          </p>
          <h1>
            Card data.
            <br />
            <span className="accent">Open to everyone.</span>
          </h1>
          <p className="hero-description">
            A free, independently curated catalogue for trading cards. Built for the things you’ll
            build next.
          </p>
          <div className="button-row">
            <Link to="/docs" className="button button-primary">
              Start building <ArrowUpRight size={18} aria-hidden="true" />
            </Link>
            <Link to="/games" className="button button-secondary">
              Explore the catalogue <ArrowRight size={18} aria-hidden="true" />
            </Link>
          </div>
          <p className="hero-footnote">
            <Check size={14} aria-hidden="true" /> No API keys <span>·</span> No subscriptions{" "}
            <span>·</span> Open source
          </p>
        </div>
      </section>
      <section className="home-workbench" aria-label="Explore the catalogue and API">
        <div className="catalogue-preview">
          <p className="eyebrow">ONE CATALOGUE. EVERY POSSIBILITY.</p>
          <h2>
            From the collection
            <br />
            to your next creation.
          </h2>
          <p>
            Browse independently sourced sets, explore individual printings, or plug the same data
            into your project.
          </p>
          <div className="preview-records">
            {featured.isPending ? (
              <output>Loading the catalogue…</output>
            ) : featured.isError ? (
              <p>
                Catalogue preview unavailable.{" "}
                <button type="button" onClick={() => void featured.refetch()}>
                  Try again
                </button>
              </p>
            ) : featured.data?.data.length ? (
              featured.data.data.map((set) => (
                <Link key={set.id} to={`/games/pokemon/sets/${set.id}`} className="preview-record">
                  <span className="preview-record-icon" aria-hidden="true">
                    <Database size={19} />
                  </span>
                  <span>
                    <strong>{set.name}</strong>
                    <small>
                      {set.synthetic
                        ? "Synthetic development fixture"
                        : "Pokémon · Published catalogue"}
                    </small>
                  </span>
                  <ArrowUpRight size={17} aria-hidden="true" />
                </Link>
              ))
            ) : (
              <p>No sets published yet. Reviewed contributions will appear here.</p>
            )}
          </div>
          <Link to="/games" className="text-link">
            Explore all sets <ArrowRight size={16} aria-hidden="true" />
          </Link>
        </div>
        <div className="hero-terminal">
          <div className="terminal-top">
            <span>
              <Terminal size={16} aria-hidden="true" /> A SIMPLE PLACE TO START
            </span>
            <span className="badge">API coming soon</span>
          </div>
          <div className="terminal-body">
            <p className="code-comment">{"// Your next project starts here."}</p>
            <p className="code-request">
              <span>GET</span> Catalogue snapshot
            </p>
            <pre aria-label="Catalogue snapshot excerpt">
              <code>{exampleText}</code>
            </pre>
            <p className="code-comment">Loaded in your browser · selected catalogue fields</p>
          </div>
          <div className="terminal-bottom">
            <span className="protocol-dot" /> Public data. Ready to build.
            <Link to="/docs" aria-label="Read API documentation">
              <ArrowUpRight size={18} aria-hidden="true" />
            </Link>
          </div>
        </div>
      </section>
      <section className="metrics-strip" aria-label="Published catalogue counts">
        {[
          ["Games", stats.data?.games],
          ["Sets", stats.data?.sets],
          ["Card identities", stats.data?.cards],
          ["Printings", stats.data?.printings],
        ].map(([label, value]) => (
          <div key={label}>
            <strong>{typeof value === "number" ? number.format(value) : "—"}</strong>
            <span>{label}</span>
          </div>
        ))}
        <Link to="/coverage">
          See measured coverage <ArrowUpRight size={16} aria-hidden="true" />
        </Link>
      </section>
      <p className="metrics-note">
        {stats.isError
          ? "Catalogue counts are temporarily unavailable."
          : stats.data?.syntheticRecords
            ? `${number.format(stats.data.syntheticRecords)} records are clearly marked synthetic development fixtures. No claim of catalogue completeness.`
            : "Counts come directly from the catalogue. Coverage is a work in progress."}
      </p>
      <section className="section">
        <div className="section-heading">
          <div>
            <p className="eyebrow">GOOD FOUNDATIONS. OPEN POSSIBILITIES.</p>
            <h2>
              Less gatekeeping.
              <br />
              More building.
            </h2>
          </div>
          <p>
            Public data infrastructure for developers,
            <br className="desktop-only" /> researchers, and the card community.
          </p>
        </div>
        <div className="feature-grid">
          <article>
            <span className="feature-icon">
              <Braces aria-hidden="true" />
            </span>
            <h3>Made for developers</h3>
            <p>
              Downloadable JSON and stable identities. The Rust API is coming soon at
              api.cardlattice.com. Small details that make a difference.
            </p>
            <Link to="/docs" className="text-link">
              API coming soon <ArrowRight size={16} aria-hidden="true" />
            </Link>
          </article>
          <article>
            <span className="feature-icon">
              <Database aria-hidden="true" />
            </span>
            <h3>Data you can trace</h3>
            <p>
              Original contributions with recorded provenance. Card metadata and artwork rights are
              always treated separately.
            </p>
            <Link to="/licensing" className="text-link">
              Our sourcing principles <ArrowRight size={16} aria-hidden="true" />
            </Link>
          </article>
          <article>
            <span className="feature-icon">
              <GitPullRequest aria-hidden="true" />
            </span>
            <h3>Better, together</h3>
            <p>
              Every correction matters. Help build a careful, independently maintained resource with
              a human review process.
            </p>
            <Link to="/contribute" className="text-link">
              Make a contribution <ArrowRight size={16} aria-hidden="true" />
            </Link>
          </article>
        </div>
      </section>
      <section className="quickstart">
        <div>
          <p className="eyebrow">NO SIGN-UP. JUST A REQUEST.</p>
          <h2>Download the manifest.</h2>
          <p>One request. No account needed.</p>
        </div>
        <div>
          <div className="copy-command">
            <code>{command}</code>
            <Button onClick={() => void copy()} aria-label="Copy curl command">
              {copied ? "Copied" : "Copy"}
            </Button>
          </div>
          <output className="field-hint">
            {copyError
              ? "Clipboard unavailable. Select and copy the command above."
              : copied
                ? "Command copied to clipboard."
                : "Static catalogue downloads need no token."}
          </output>
        </div>
      </section>
    </div>
  );
}
