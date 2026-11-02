import { useApi, date } from "../api";
import { PageHeader } from "../components/PageHeader";
import { QueryState } from "../components/QueryState";

export function Changelog() {
  const query = useApi<{
    data: { revision: number; name: string; kind: string; createdAt: string; operation: string }[];
  }>("/v1/changelog");
  return (
    <div className="page">
      <PageHeader eyebrow="A PUBLIC RECORD OF CHANGE" title="The catalogue keeps growing.">
        <p>The latest 50 published metadata revisions, from the audit log.</p>
      </PageHeader>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {query.data?.data.length ? (
          <ol className="timeline">
            {query.data.data.map((change) => (
              <li key={change.revision}>
                <span className="timeline-revision">r{change.revision}</span>
                <div>
                  <p className="eyebrow">
                    {change.operation} · {change.kind}
                  </p>
                  <h2>{change.name}</h2>
                  <time dateTime={change.createdAt}>{date(change.createdAt)}</time>
                </div>
              </li>
            ))}
          </ol>
        ) : (
          <div className="empty">
            <h2>No revisions yet</h2>
            <p>Approved contributions will appear here.</p>
          </div>
        )}
      </QueryState>
    </div>
  );
}
