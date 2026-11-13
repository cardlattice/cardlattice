import { PageHeader } from "../components/PageHeader";
export function Changelog() {
  return (
    <div className="page narrow">
      <PageHeader eyebrow="A PUBLIC RECORD OF CHANGE" title="The catalogue keeps growing.">
        <p>Reviewed data changes and source history live in the public repository.</p>
      </PageHeader>
      <a
        className="button button-primary"
        href="https://github.com/cardlattice/cardlattice-data/commits/main/"
      >
        View catalogue history ↗
      </a>
    </div>
  );
}
