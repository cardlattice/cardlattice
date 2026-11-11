import { Link, useRouterState } from "@tanstack/react-router";
import { PageHeader } from "../components/PageHeader";

export function Information() {
  const licensing = useRouterState({ select: (state) => state.location.pathname === "/licensing" });
  return (
    <div className="page narrow">
      <PageHeader
        eyebrow={licensing ? "KNOW WHERE YOUR DATA COMES FROM" : "PUBLIC DATA INFRASTRUCTURE"}
        title={licensing ? "Open, with clear boundaries." : "A shared foundation for card data."}
      >
        <p>
          {licensing
            ? "Metadata, software, and artwork have different rights. We keep those distinctions visible."
            : "Cardlattice is a free, open-source, independently maintained trading card catalogue and public API."}
        </p>
      </PageHeader>
      <div className="prose">
        {licensing ? (
          <>
            <h2>Software & original metadata</h2>
            <p>
              Cardlattice software is licensed under MIT. Original metadata contributions and the
              synthetic development fixtures are dedicated under CC0-1.0. Legally required notices
              and attribution must always be preserved.
            </p>
            <h2>Independent sourcing</h2>
            <p>
              Accepted sources are original manual entry, inspection of user-owned physical cards,
              permitted official references, and original community contributions. We do not seed
              the catalogue with TCGdex, Scryfall, YGOPRODeck, or other third-party catalogue dumps
              or API responses.
            </p>
            <p>
              Each reviewed change records its source and field-level provenance internally. Source
              descriptions are visible on record pages. Do not include private personal information
              in a source description.
            </p>
            <h2>Artwork is separate</h2>
            <p>
              Owning a card, photographing it, or scanning it does not grant permission to
              redistribute its artwork. Uploaded front and back scans are stored privately for human
              review. Metadata approval does not approve the images for publication.
            </p>
            <p>
              Public images require separate documented permission or a suitable licence, a rights
              review, and any required attribution. Development fixtures include no third-party
              artwork.
            </p>
            <h2>Contributor terms</h2>
            <ol>
              <li>
                Contribute only metadata you independently collected or are authorised to provide.
              </li>
              <li>
                Dedicate your original factual metadata contribution under CC0-1.0 to the extent
                legally possible. Do not submit copied descriptive text from protected sources.
              </li>
              <li>
                Give an accurate source description and identify relevant rights restrictions.
              </li>
              <li>
                Upload only scans you are authorised to provide privately for review. You grant the
                project permission to store and inspect them for that purpose; no public artwork
                licence is implied.
              </li>
              <li>
                Accept human review, duplicate checks, corrections, rejection, and an audit history.
                Do not include unnecessary personal information.
              </li>
            </ol>
            <h2>Corrections & removal requests</h2>
            <p>
              Use the contribution form to propose corrections or request removal of a record. A
              maintainer must review the source and preserve the audit trail. Production operators
              must publish a monitored rights-contact address before launching publicly.
            </p>
          </>
        ) : (
          <>
            <h2>Free access is the foundation.</h2>
            <p>
              Public catalogue reads require no API key, subscription, or registration. The
              application runs with PostgreSQL, a Rust API, and a static React website. No paid
              service or proprietary infrastructure is required.
            </p>
            <h2>A catalogue that earns your trust.</h2>
            <p>
              We start with Pokémon, while keeping the common model open to other games. Card
              identities, releases, and printings are distinct. Game-specific attributes retain
              their own validation rules.
            </p>
            <p>
              We never claim completeness without a verified reference total. The{" "}
              <Link to="/coverage">coverage dashboard</Link> reports counts and known gaps from the
              actual database. Synthetic development fixtures are clearly labelled.
            </p>
            <h2>Independent governance</h2>
            <p>
              This is an independently maintained open-source project, unaffiliated with card
              publishers. Maintainers steward code, source policy, moderation access, and operating
              costs. Changes should be reviewed publicly; significant data-policy decisions should
              document their rationale.
            </p>
            <p>
              Contributors can propose code changes, new game adapters, original metadata, and
              corrections. Moderators approve factual metadata with a recorded reason. Rights
              decisions for artwork remain a separate review.
            </p>
            <h2>Built to stay practical.</h2>
            <p>
              A single API process, PostgreSQL-backed durable jobs, immutable dataset exports, and
              optional CDN caching keep the architecture understandable. Expensive third-party
              dependencies are not required.
            </p>
            <div className="button-row">
              <Link to="/contribute" className="button button-primary">
                Contribute to the catalogue →
              </Link>
              <Link to="/docs" className="button button-secondary">
                Read the API docs
              </Link>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
