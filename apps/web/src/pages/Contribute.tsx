import { useSearch } from "@tanstack/react-router";
import { useApi, type DetailData } from "../api";
import { PageHeader } from "../components/PageHeader";
import { QueryState } from "../components/QueryState";
import { ContributionForm } from "../components/ContributionForm";

export function Contribute() {
  const search = useSearch({ strict: false }) as { target?: string; kind?: string; game?: string };
  const target = search.target;
  const kind = ["card", "set", "printing", "series", "product", "collection", "artwork"].includes(
    search.kind || "",
  )
    ? search.kind!
    : "card";
  const original = useApi<DetailData>(
    `/v1/games/${search.game || "pokemon"}/${kind === "series" ? "series" : `${kind}s`}/${target}`,
    !!target,
  );
  return (
    <div className="page narrow">
      <PageHeader
        eyebrow="GROW THE COMMONS"
        title={target ? "Make the record better." : "Good data starts with a source."}
      >
        <p>
          Submit a new record or correction. A moderator reviews every change before it reaches the
          public catalogue.
        </p>
      </PageHeader>
      <aside className="notice">
        <strong>Public reads are always anonymous.</strong>
        <p>
          Submissions use a maintainer-issued contributor token to limit abuse. In local
          development, retrieve yours with{" "}
          <code>docker compose exec api cat /run/cardlattice/contributor</code>. Tokens are used for
          this visit only.
        </p>
      </aside>
      {target ? (
        <QueryState pending={original.isPending} error={original.error} retry={original.refetch}>
          {original.data && (
            <ContributionForm
              key={target}
              original={original.data.data}
              initialGame={search.game}
            />
          )}
        </QueryState>
      ) : (
        <ContributionForm initialGame={search.game} />
      )}
    </div>
  );
}
