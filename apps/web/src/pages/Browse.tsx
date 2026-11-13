import { useCatalogue } from "../catalogue";
import { Link, useNavigate, useParams, useRouterState, useSearch } from "@tanstack/react-router";
import { Search, ArrowRight } from "lucide-react";
import { useState, type FormEvent } from "react";
import { type PageData, type RecordData } from "../api";
import { PageHeader } from "../components/PageHeader";
import { RecordTable } from "../components/RecordTable";
import { QueryState } from "../components/QueryState";
import { Button } from "../components/Button";

export function Browse() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const params = useParams({ strict: false }) as { game?: string };
  const search = useSearch({ strict: false }) as {
    q?: string;
    cursor?: string;
    language?: string;
    sort?: string;
    setId?: string;
    cardId?: string;
    productId?: string;
    collectionId?: string;
    artworkId?: string;
  };
  const navigate = useNavigate();
  const isSearch = pathname === "/search";
  const isGames = pathname === "/games";
  const isGame = !!params.game && pathname === `/games/${params.game}`;
  const collection = pathname.split("/")[3] || "sets";
  const [draftSearch, setDraftSearch] = useState(search.q || "");
  const query = new URLSearchParams();
  if (search.q) query.set("q", search.q);
  if (search.cursor) query.set("cursor", search.cursor);
  if (search.language) query.set("language", search.language);
  if (search.setId) query.set("setId", search.setId);
  if (search.cardId) query.set("cardId", search.cardId);
  if (search.productId) query.set("productId", search.productId);
  if (search.collectionId) query.set("collectionId", search.collectionId);
  if (search.artworkId) query.set("artworkId", search.artworkId);
  if (search.sort === "newest") query.set("sort", "newest");
  query.sort();
  const endpoint = isSearch
    ? "/search"
    : isGames
      ? "/games"
      : `/games/${params.game}/${collection}`;
  const records = useCatalogue<PageData>(`${endpoint}${query.size ? `?${query}` : ""}`);
  const game = useCatalogue<{ data: RecordData }>(`/games/${params.game}`, !!params.game);
  const parent = useCatalogue<{ data: RecordData }>(
    `/games/${params.game}/${search.setId ? "sets" : "cards"}/${search.setId || search.cardId}`,
    !!params.game && !!(search.setId || search.cardId),
  );
  const title = isSearch
    ? "Find your next starting point."
    : isGames
      ? "A catalogue, built in the open."
      : isGame
        ? game.data?.data.name || "Game catalogue"
        : collection === "sets"
          ? "Browse sets."
          : collection === "cards"
            ? "Card identities."
            : collection === "products"
              ? "Distribution products."
              : collection === "collections"
                ? "Collector collections."
                : collection === "artworks"
                  ? "Shared illustrations."
                  : parent.data?.data.name || "Every printing has a story.";
  function update(values: Record<string, string>) {
    void navigate({ to: pathname, search: { ...search, cursor: "", ...values } });
  }
  function submit(event: FormEvent) {
    event.preventDefault();
    update({ q: draftSearch });
  }
  return (
    <div className="page">
      <PageHeader eyebrow={isSearch ? "GLOBAL SEARCH" : "EXPLORE THE CATALOGUE"} title={title}>
        <p>
          {isSearch
            ? "Search names across games, sets, cards, printings, and localised metadata."
            : "Independently sourced metadata. Distinct identities. Room to grow."}
        </p>
      </PageHeader>
      {params.game && (
        <nav className="tabs" aria-label="Game resources">
          {["sets", "cards", "printings"].map((tab) => (
            <Link
              key={tab}
              to={`/games/${params.game}/${tab}`}
              className={collection === tab ? "selected" : ""}
            >
              {tab[0].toUpperCase() + tab.slice(1)}
            </Link>
          ))}
        </nav>
      )}
      <div className="browse-toolbar">
        <form className="search-form" onSubmit={submit}>
          <label htmlFor="catalogue-search" className="sr-only">
            Search catalogue
          </label>
          <Search size={19} aria-hidden="true" />
          <input
            id="catalogue-search"
            value={draftSearch}
            onChange={(event) => setDraftSearch(event.target.value)}
            maxLength={120}
            placeholder="Search card names, sets, and more…"
          />
          <Button type="submit">Search</Button>
        </form>
        <label className="inline-field">
          Sort
          <select
            value={search.sort || "name"}
            onChange={(event) => update({ sort: event.target.value })}
          >
            <option value="name">Name A–Z</option>
            <option value="newest">Newest first</option>
          </select>
        </label>
        <label className="inline-field">
          Language
          <select
            value={search.language || ""}
            onChange={(event) => update({ language: event.target.value })}
          >
            <option value="">All languages</option>
            <option value="en">English</option>
            <option value="ja">Japanese</option>
            <option value="nl">Dutch</option>
            <option value="ko">Korean</option>
            <option value="zh-Hans">Simplified Chinese</option>
            <option value="fr">French</option>
            <option value="de">German</option>
          </select>
        </label>
      </div>
      <QueryState pending={records.isPending} error={records.error} retry={records.refetch}>
        <div className="list-heading">
          <span>
            {isGames ? "SUPPORTED GAMES" : isSearch ? "SEARCH RESULTS" : collection.toUpperCase()}
          </span>
          <span>{records.data?.data.length ?? 0} on this page</span>
        </div>
        <RecordTable records={records.data?.data || []} game={params.game} />
        <div className="pagination">
          {search.cursor && <Button onClick={() => update({ cursor: "" })}>First page</Button>}
          {records.data?.nextCursor && (
            <Button onClick={() => update({ cursor: records.data!.nextCursor! })}>
              Next page <ArrowRight size={16} aria-hidden="true" />
            </Button>
          )}
        </div>
      </QueryState>
      <aside className="notice">
        <strong>A growing catalogue, with honest coverage.</strong>
        <p>
          Synthetic fixtures are labelled throughout. Game support doesn’t imply a complete
          catalogue. <Link to="/coverage">View coverage</Link>.
        </p>
      </aside>
    </div>
  );
}
