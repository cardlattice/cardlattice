import { PageHeader } from "../components/PageHeader";
export function Contribute() {
  return (
    <div className="page narrow">
      <PageHeader eyebrow="GROW THE COMMONS" title="Good data starts with a source.">
        <p>Propose original metadata and corrections through the data repository.</p>
      </PageHeader>
      <section className="panel prose">
        <h2>Contribute on GitHub</h2>
        <p>
          Edit the card’s YAML file and open a pull request. A maintainer reviews changes before
          they enter the published catalogue.
        </p>
        <a
          className="button button-primary"
          href="https://github.com/cardlattice/cardlattice-data/blob/main/CONTRIBUTING.md"
        >
          Contribution guide ↗
        </a>
        <p>
          Online submission and moderation will be available when the Rust API launches. Please use
          GitHub for now.
        </p>
      </section>
    </div>
  );
}
