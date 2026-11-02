import { useGetV1Stats } from "../generated/catalogue";
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
import { number, type StatsData } from "../api";
import { Button } from "../components/Button";

export function Home() {
  const stats = useGetV1Stats<StatsData, Error>({ query: { staleTime: 30_000, retry: 1 } });
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const command = "curl http://localhost:8080/v1/games/pokemon/cards";
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
            Without the
            <br /> <span className="accent">barriers.</span>
          </h1>
          <p className="hero-description">
            A free, independently curated catalogue and public API for trading cards. Built for the
            things you’ll build next.
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
        <div className="hero-terminal">
          <div className="terminal-top">
            <span>
              <Terminal size={16} aria-hidden="true" /> A SIMPLE PLACE TO START
            </span>
            <span className="badge">REST / v1</span>
          </div>
          <div className="terminal-body">
            <p className="code-comment">{"// Your next project starts here."}</p>
            <p className="code-request">
              <span>GET</span> /v1/games/pokemon/cards
            </p>
            <pre aria-label="Illustrative API response">
              <code>
                {
                  '{\n  "data": [\n    {\n      "kind": "card",\n      "name": "Prism Seed — synthetic",\n      "synthetic": true,\n      "attributes": {\n        "category": "pokemon",\n        "hp": 40\n      }\n    }\n  ],\n  "nextCursor": null,\n  "limit": 20\n}'
                }
              </code>
            </pre>
            <p className="code-comment">Illustrative excerpt · original development fixture</p>
          </div>
          <div className="terminal-bottom">
            <span className="protocol-dot" /> JSON in. Possibilities out.
            <Link to="/docs" aria-label="Read API documentation">
              <ArrowUpRight size={18} aria-hidden="true" />
            </Link>
          </div>
        </div>
      </section>
      <section className="metrics-strip" aria-label="Live catalogue counts">
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
          ? "Live counts are temporarily unavailable."
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
              Predictable JSON, stable identities, cursor pagination, and an open API contract.
              Small details that make a difference.
            </p>
            <Link to="/docs" className="text-link">
              Explore the API <ArrowRight size={16} aria-hidden="true" />
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
          <h2>Try your first query.</h2>
          <p>Run Cardlattice locally, then open a terminal.</p>
        </div>
        <div>
          <div className="copy-command">
            <code>{command}</code>
            <Button onClick={() => void copy()} aria-label="Copy curl command">
              {copied ? "Copied" : "Copy"}
            </Button>
          </div>
          <p className="field-hint" role="status">
            {copyError
              ? "Clipboard unavailable. Select and copy the command above."
              : copied
                ? "Command copied to clipboard."
                : "Public reads work without a token."}
          </p>
        </div>
      </section>
    </div>
  );
}
