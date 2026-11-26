import { useCatalogue } from "./catalogue-provider";
import { number } from "./data";
import { catalogueBase } from "./snapshot";
import { Link } from "@tanstack/react-router";
import * as stylex from "@stylexjs/stylex";
import { Button } from "@imlunahey/components/button";
import { Panel } from "@imlunahey/components/panel";
import { Badge } from "@imlunahey/components/badge";
import { DataTable } from "@imlunahey/components/data-table";
import { Progress } from "@imlunahey/components/progress";
import { CodeBlock } from "@imlunahey/components/code-block";
import { DescriptionList } from "@imlunahey/components/description-list";

import { s } from "./styles";
export function Information({ page }: { page: string }) {
  const { sets, printings, manifest, sourceRecords } = useCatalogue();
  function download() {
    const url = URL.createObjectURL(
      new Blob(
        [
          JSON.stringify(
            {
              schemaVersion: manifest.schemaVersion,
              sourceCommit: manifest.sourceCommit,
              records: sourceRecords,
            },
            null,
            2,
          ),
        ],
        { type: "application/json" },
      ),
    );
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "cardlattice-catalogue.json";
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  if (page === "coverage") {
    const recorded = printings.filter((card) => card.artist).length;
    const coverage = sets.map((set) => ({
      name: set.name,
      total: printings.filter((card) => card.setId === set.id).length,
      artists: printings.filter((card) => card.setId === set.id && card.artist).length,
    }));
    return (
      <>
        <span {...stylex.props(s.eyebrow)}>Know what’s here. See what’s next.</span>
        <h1 {...stylex.props(s.title)}>Catalogue coverage</h1>
        <p {...stylex.props(s.lead)}>
          Counts and missing fields come from the published snapshot. Set totals are recorded
          printings, not estimates of how complete a set is.
        </p>
        <div {...stylex.props(s.three)}>
          <Panel title="Recorded printings">
            <span {...stylex.props(s.stat)}>{number(printings.length)}</span>
            <p {...stylex.props(s.muted)}>Across {sets.length} sets</p>
          </Panel>
          <Panel title="Illustrators recorded">
            <Progress
              label="Illustrators"
              value={recorded}
              max={printings.length}
              detail={`${recorded} of ${printings.length} published printings`}
            />
          </Panel>
          <Panel title="Open fields">
            <span {...stylex.props(s.stat)}>{printings.length - recorded}</span>
            <p {...stylex.props(s.muted)}>Illustrator details still to record</p>
          </Panel>
        </div>
        <DataTable
          label="Published catalogue coverage"
          data={coverage}
          columns={[
            { accessorKey: "name", header: "Set" },
            { accessorKey: "total", header: "Recorded printings" },
            { accessorKey: "artists", header: "Illustrators recorded" },
            {
              id: "missing",
              header: "Missing",
              cell: ({ row }) => row.original.total - row.original.artists,
            },
          ]}
        />
        <Link to="/contribute" {...stylex.props(s.link)}>
          Help fill a gap →
        </Link>
      </>
    );
  }
  if (page === "data")
    return (
      <div {...stylex.props(s.narrow)}>
        <span {...stylex.props(s.eyebrow)}>Build something of your own</span>
        <h1 {...stylex.props(s.title)}>Catalogue data</h1>
        <p {...stylex.props(s.lead)}>
          Download exactly what powers the catalogue. No account or API key needed.
        </p>
        <Panel title="Verified catalogue snapshot" actions={<Badge tone="neutral">JSON</Badge>}>
          <DescriptionList
            items={[
              { label: "Sets", value: sets.length },
              { label: "Printings", value: printings.length },
              {
                label: "Includes",
                value: "Published metadata · artwork loaded separately from TCGdex",
              },
              { label: "Source", value: "Cardlattice published snapshot" },
              { label: "Revision", value: manifest.sourceCommit.slice(0, 12) },
            ]}
          />
          <Button variant="primary" onClick={download}>
            Download catalogue JSON
          </Button>
        </Panel>
        <CodeBlock
          label="Snapshot shape"
          code={JSON.stringify(
            {
              schemaVersion: manifest.schemaVersion,
              sourceCommit: manifest.sourceCommit,
              records: sourceRecords.filter((r) => r.metadata.kind === "printing").slice(0, 1),
            },
            null,
            2,
          )}
        />
        <p {...stylex.props(s.muted)}>
          Download{" "}
          <a href={catalogueBase + manifest.filename} {...stylex.props(s.link)}>
            the original verified snapshot ↗
          </a>
          . The public query API is coming soon.
        </p>
      </div>
    );
  return null;
}
