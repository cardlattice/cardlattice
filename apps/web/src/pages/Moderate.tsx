import { PageHeader } from "../components/PageHeader";
export function Moderate() {
  return (
    <div className="page narrow">
      <PageHeader eyebrow="API COMING SOON" title="Online moderation is coming soon.">
        <p>
          Until the Rust API launches, catalogue changes are reviewed through GitHub pull requests.
        </p>
      </PageHeader>
      <a
        className="button button-primary"
        href="https://github.com/cardlattice/cardlattice-data/pulls"
      >
        Review catalogue pull requests ↗
      </a>
    </div>
  );
}
