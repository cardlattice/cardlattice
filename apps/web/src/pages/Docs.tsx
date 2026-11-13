import { Link } from "@tanstack/react-router";
import { PageHeader } from "../components/PageHeader";
import { manifestUrl } from "../catalogue-data";
export function Docs() {
  return (
    <div className="page narrow">
      <PageHeader eyebrow="BUILD WITH OPEN DATA" title="The API is coming soon.">
        <p>
          The Rust server will be available at <strong>api.cardlattice.com</strong>. Public API
          requests and online submissions are not available yet.
        </p>
      </PageHeader>
      <section className="panel prose">
        <h2>Use the catalogue today</h2>
        <p>
          Download the published JSON snapshot, or browse it here. The app loads the data into your
          browser, where search and filtering run locally.
        </p>
        <pre className="code-block">
          <code>{`curl "${manifestUrl}"`}</code>
        </pre>
        <p>
          The manifest names the snapshot file and includes its SHA-256 checksum, byte size, source
          revision and record counts.
        </p>
        <Link to="/datasets" className="button button-primary">
          Download catalogue data
        </Link>
        <p>
          <a href="https://github.com/cardlattice/cardlattice/blob/main/docs/openapi.json">
            Preview the planned API contract on GitHub ↗
          </a>
        </p>
      </section>
    </div>
  );
}
