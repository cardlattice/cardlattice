import { Link, useParams, useRouterState } from "@tanstack/react-router";
import { useApi, date, type DetailData } from "../api";
import { QueryState } from "../components/QueryState";
import { PageHeader } from "../components/PageHeader";
import { RecordTable } from "../components/RecordTable";
import { SourceCitation } from "../components/SourceCitation";

export function Detail() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const { game } = useParams({ strict: false }) as { game: string; id: string };
  const query = useApi<DetailData>(`/v1${pathname}`);
  const record = query.data?.data;
  return (
    <div className="page">
      <Link to={`/games/${game}`} className="text-link">
        ← Back to game
      </Link>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {record && (
          <>
            <PageHeader
              eyebrow={record.kind.toUpperCase()}
              title={record.name}
              action={
                <Link
                  to="/contribute"
                  search={{ target: record.id, kind: record.kind, game }}
                  className="button button-secondary"
                >
                  Suggest a correction
                </Link>
              }
            >
              <p>
                {record.synthetic
                  ? "Original synthetic development fixture. This is not a real card product."
                  : "Independently reviewed catalogue metadata."}
              </p>
            </PageHeader>
            <div className="detail-grid">
              <section className="panel">
                <h2>Record metadata</h2>
                <dl className="metadata">
                  {Object.entries({
                    "Internal ID": record.id,
                    "Collector number": record.collectorNumber,
                    Language: record.language,
                    Region: record.region,
                    Rarity: record.rarity,
                    "Release date": record.releaseDate,
                    "Last updated": date(record.updatedAt),
                  }).map(([label, value]) => (
                    <div key={label}>
                      <dt>{label}</dt>
                      <dd>{value || "Not recorded"}</dd>
                    </div>
                  ))}
                </dl>
                {record.cardId && (
                  <p>
                    <Link to={`/games/${game}/cards/${record.cardId}`}>View card identity →</Link>
                  </p>
                )}
                {record.setId && (
                  <p>
                    <Link to={`/games/${game}/sets/${record.setId}`}>View set →</Link>
                  </p>
                )}
              </section>
              <section className="panel">
                <h2>Game-specific attributes</h2>
                {!!record.attributes.identityScope && (
                  <p className="notice">
                    This card identity is provisional. It remains separate from same-name cards
                    until their rules and characteristics have been compared.
                  </p>
                )}
                <pre className="code-block">
                  <code>{JSON.stringify(record.attributes, null, 2)}</code>
                </pre>
                <h3>Artwork</h3>
                {query.data?.images.length ? (
                  query.data.images.map((image) => (
                    <figure key={image.id}>
                      <img src={image.url} alt={`${record.name}, ${image.side}`} loading="lazy" />
                      <figcaption>
                        {image.license} · {image.attribution}
                      </figcaption>
                    </figure>
                  ))
                ) : (
                  <p className="muted">
                    No publication-cleared artwork is available. Owning a card or scan does not
                    grant artwork redistribution rights.
                  </p>
                )}
                {!!query.data?.variants.length && (
                  <>
                    <h3>Printing variants</h3>
                    <ul>
                      {query.data.variants.map((variant) => (
                        <li key={variant.id}>
                          {variant.finish} {variant.stamp}
                        </li>
                      ))}
                    </ul>
                  </>
                )}
              </section>
            </div>
            {["card", "set"].includes(record.kind) && (
              <section className="section">
                <div className="section-heading">
                  <h2>Related printings</h2>
                  <Link
                    to={`/games/${game}/printings`}
                    search={record.kind === "set" ? { setId: record.id } : { cardId: record.id }}
                    className="text-link"
                  >
                    Browse printings →
                  </Link>
                </div>
                <RecordTable records={query.data?.printings || []} game={game} />
                <p className="field-hint">
                  Up to 20 related printings.{" "}
                  <a href={query.data?.printingsUrl || "#"}>Paginated API collection</a>
                </p>
              </section>
            )}
            <section className="section">
              <h2>Sources & provenance</h2>
              {query.data?.sources.map((source) => (
                <SourceCitation key={source.id} source={source} />
              ))}
              {!!query.data?.sources.length && (
                <p className="field-hint">
                  <a href={`/v1${pathname}`}>Full provenance (JSON)</a>
                </p>
              )}
              {query.data?.localisations.map((locale) => (
                <p key={locale.language}>
                  <span className="badge">{locale.language}</span> {locale.name}
                </p>
              ))}
            </section>
          </>
        )}
      </QueryState>
    </div>
  );
}
