import { useState } from "react";
import { useNavigate, useRouterState } from "@tanstack/react-router";
import { SearchField } from "@imlunahey/components/search-field";
export function SearchForm() {
  const query = useRouterState({
    select: (state) => String((state.location.search as Record<string, unknown>).q ?? ""),
  });
  return <CatalogueSearch key={query} query={query} />;
}
function CatalogueSearch({ query }: { query: string }) {
  // This component is keyed by query above, so URL changes reset the editable draft.
  // oxlint-disable-next-line react-doctor/no-derived-useState
  const [value, setValue] = useState(query);
  const navigate = useNavigate();
  return (
    <SearchField
      label="Search cards"
      value={value}
      onValueChange={setValue}
      onSearch={(q) => void navigate({ to: "/search", search: { q: q.trim() } })}
      placeholder="Search cards or sets…"
    />
  );
}
