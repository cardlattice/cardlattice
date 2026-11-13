import { useCatalogue } from "../catalogue";
import { Link } from "@tanstack/react-router";
import { number, type StatsData } from "../api";
import { PageHeader } from "../components/PageHeader";
import { QueryState } from "../components/QueryState";

export function Coverage() {
  const query = useCatalogue<StatsData>("/stats");
  const stats = query.data;
  return (
    <div className="page">
      <PageHeader eyebrow="MEASURED, NOT ASSUMED" title="Every record counts.">
        <p>What’s present, what’s missing, and what we don’t yet know.</p>
      </PageHeader>
      <QueryState pending={query.isPending} error={query.error} retry={query.refetch}>
        {stats && (
          <>
            <section className="metrics-strip">
              {[
                ["Games", stats.games],
                ["Sets", stats.sets],
                ["Cards", stats.cards],
                ["Printings", stats.printings],
              ].map(([label, value]) => (
                <div key={label}>
                  <strong>{number.format(Number(value))}</strong>
                  <span>{label}</span>
                </div>
              ))}
            </section>
            <section className="detail-grid section">
              <article className="panel">
                <h2>Metadata coverage</h2>
                <p>Calculated from the {stats.printings} printings in the published snapshot.</p>
                <label className="coverage-label" htmlFor="rarity-coverage">
                  Rarity recorded{" "}
                  <strong>
                    {stats.printings - stats.missingRarity} / {stats.printings}
                  </strong>
                </label>
                <progress
                  id="rarity-coverage"
                  max={stats.printings || 1}
                  value={stats.printings - stats.missingRarity}
                />
                <label className="coverage-label" htmlFor="image-coverage">
                  Publication-cleared image{" "}
                  <strong>
                    {stats.printingsWithImages} / {stats.printings}
                  </strong>
                </label>
                <progress
                  id="image-coverage"
                  max={stats.printings || 1}
                  value={stats.printingsWithImages}
                />
              </article>
              <article className="panel">
                <h2>Catalogue completeness</h2>
                <p className="large-value">Unknown</p>
                <p>{stats.completenessNote}</p>
                <p>
                  <a href="https://status.cardlattice.com">View coverage by language and set ↗</a>
                </p>
                <p className="field-hint">
                  Counts are calculated in your browser from the published snapshot.
                </p>
              </article>
            </section>
            <aside className="notice">
              <strong>Coverage is a shared effort.</strong>
              <p>
                Help fill a gap with original, traceable metadata.{" "}
                <Link to="/contribute">Make a contribution →</Link>
              </p>
            </aside>
          </>
        )}
      </QueryState>
    </div>
  );
}
