import { Download } from "lucide-react";
import { useApi, date, number } from "../api";
import { QueryState } from "../components/QueryState";
import { PageHeader } from "../components/PageHeader";

type Manifest = {
  id: string;
  revision: number;
  schemaVersion: number;
  sha256: string;
  byteSize: number;
  recordCount: number;
  filename: string;
  createdAt: string;
};
export function Datasets() {
  const query = useApi<{ data: Manifest[] }>("/v1/datasets");
  return (
    <div className="page">
      <PageHeader eyebrow="TAKE THE DATA WITH YOU" title="Open data. In one download.">
        <p>
          Immutable, versioned metadata snapshots. No account, no artwork, no hidden dependencies.
        </p>
      </PageHeader>
      <div className="notice">
        <strong>CC0-1.0 · Newline-delimited JSON</strong>
        <p>
          Each line is a catalogue record. Verify the SHA-256 before importing. Synthetic records
          retain their explicit flag. The latest 50 manifests are listed here; older URLs remain
          immutable.
        </p>
      </div>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {query.data?.data.length ? (
          query.data.data.map((manifest) => (
            <article className="dataset panel" key={manifest.id}>
              <div>
                <p className="eyebrow">
                  SCHEMA v{manifest.schemaVersion} · {date(manifest.createdAt)}
                </p>
                <h2>Catalogue revision {manifest.revision}</h2>
                <p>
                  {number.format(manifest.recordCount)} records · {number.format(manifest.byteSize)}{" "}
                  bytes
                </p>
                <details>
                  <summary>SHA-256 checksum</summary>
                  <code className="break-code">{manifest.sha256}</code>
                </details>
              </div>
              <a className="button button-primary" href={`/datasets/${manifest.filename}`} download>
                <Download size={17} aria-hidden="true" /> Download NDJSON
              </a>
            </article>
          ))
        ) : (
          <div className="empty">
            <h2>No snapshots published yet</h2>
            <p>The publication worker creates a snapshot after the first catalogue revision.</p>
          </div>
        )}
      </QueryState>
    </div>
  );
}
