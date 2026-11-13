import { Download } from "lucide-react";
import { number } from "../api";
import { useSnapshot } from "../catalogue";
import { catalogueBase, manifestUrl } from "../catalogue-data";
import { QueryState } from "../components/QueryState";
import { PageHeader } from "../components/PageHeader";
export function Datasets() {
  const query = useSnapshot();
  const manifest = query.data?.manifest;
  return (
    <div className="page">
      <PageHeader eyebrow="TAKE THE DATA WITH YOU" title="Open data. In one download.">
        <p>The same published snapshot used by this app. No account or API required.</p>
      </PageHeader>
      <div className="notice">
        <strong>CC0-1.0 · JSON</strong>
        <p>
          Original factual metadata only. Artwork is not included. Verify the SHA-256 checksum
          before importing.
        </p>
      </div>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {manifest && (
          <article className="dataset panel">
            <div>
              <p className="eyebrow">SCHEMA v{manifest.schemaVersion}</p>
              <h2>Published catalogue</h2>
              <p>
                {number.format(query.data!.records.length)} records ·{" "}
                {number.format(manifest.byteSize)} bytes
              </p>
              <p>
                Source:{" "}
                <a
                  href={`https://github.com/cardlattice/cardlattice-data/commit/${manifest.sourceCommit}`}
                >
                  {manifest.sourceCommit.slice(0, 7)}
                </a>
              </p>
              <details>
                <summary>SHA-256 checksum</summary>
                <code className="break-code">{manifest.sha256}</code>
              </details>
            </div>
            <div className="button-row">
              <a className="button button-primary" href={`${catalogueBase}${manifest.filename}`}>
                <Download size={17} aria-hidden="true" /> Download JSON
              </a>
              <a href={manifestUrl}>Manifest</a>
            </div>
          </article>
        )}
      </QueryState>
      <p>
        <a href="https://github.com/cardlattice/cardlattice-data/releases">
          Browse archived releases on GitHub ↗
        </a>
      </p>
    </div>
  );
}
