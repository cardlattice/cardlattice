import { PageHeader } from "../components/PageHeader";
export function Status() {
  return (
    <div className="page narrow">
      <PageHeader eyebrow="CATALOGUE STATUS" title="See what’s recorded.">
        <p>Our status site reports catalogue coverage by language and set.</p>
      </PageHeader>
      <a className="button button-primary" href="https://status.cardlattice.com">
        Open catalogue status ↗
      </a>
      <section className="panel section">
        <h2>API — coming soon</h2>
        <p>
          The Rust API will be deployed at api.cardlattice.com later. The app currently loads
          published data directly in your browser.
        </p>
      </section>
    </div>
  );
}
