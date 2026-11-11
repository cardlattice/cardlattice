import { useApi, date } from "../api";
import { PageHeader } from "../components/PageHeader";
import { QueryState } from "../components/QueryState";

type StatusData = {
  status: string;
  measurements: { healthy: boolean; databaseLatencyMs: number; measuredAt: string }[];
  pendingInvalidations: number;
  failedInvalidations: number;
  latestPublicationAt: string | null;
  note: string;
};
export function Status() {
  const query = useApi<StatusData>("/v1/status");
  const status = query.data;
  return (
    <div className="page">
      <PageHeader eyebrow="OBSERVED AT THE ORIGIN" title="Status, with the evidence.">
        <p>Persisted health probes, publication state, and honest gaps in measurement.</p>
      </PageHeader>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {status && (
          <>
            <section className="status-banner">
              <span className={`status-dot ${status.status}`} />
              <div>
                <h2>
                  {status.status === "operational"
                    ? "Origin responding"
                    : status.status === "degraded"
                      ? "Service degraded"
                      : "Awaiting fresh measurements"}
                </h2>
                <p>
                  {status.measurements[0]
                    ? `Latest probe: ${date(status.measurements[0].measuredAt)}`
                    : "No persisted probe is available yet."}
                </p>
              </div>
            </section>
            <figure
              className="probe-chart"
              aria-label={`${status.measurements.length} recent origin probes; ${status.measurements.filter((probe) => !probe.healthy).length} reported unhealthy`}
            >
              {[...status.measurements].reverse().map((probe) => (
                <span
                  key={probe.measuredAt}
                  className={probe.healthy ? "probe good" : "probe bad"}
                  title={`${date(probe.measuredAt)} · ${probe.databaseLatencyMs.toFixed(2)} ms · ${probe.healthy ? "healthy" : "unhealthy"}`}
                />
              ))}
            </figure>
            <p className="field-hint">
              Up to 120 stored probes · 30-second interval · older to newer
            </p>
            <div className="detail-grid section">
              <article className="panel">
                <h2>Publication & caching</h2>
                <dl className="metadata">
                  <div>
                    <dt>Pending invalidations</dt>
                    <dd>{status.pendingInvalidations}</dd>
                  </div>
                  <div>
                    <dt>Failed invalidations</dt>
                    <dd>{status.failedInvalidations}</dd>
                  </div>
                  <div>
                    <dt>Latest snapshot</dt>
                    <dd>
                      {status.latestPublicationAt
                        ? date(status.latestPublicationAt)
                        : "Not published"}
                    </dd>
                  </div>
                </dl>
              </article>
              <article className="panel">
                <h2>What this measures</h2>
                <p>{status.note}</p>
                <p>
                  This page does not claim global uptime or measure CDN hit rates. Refresh to
                  request current observations.
                </p>
                <button
                  type="button"
                  className="button button-secondary"
                  onClick={() => void query.refetch()}
                  disabled={query.isFetching}
                >
                  {query.isFetching ? "Refreshing…" : "Refresh measurements"}
                </button>
              </article>
            </div>
          </>
        )}
      </QueryState>
    </div>
  );
}
