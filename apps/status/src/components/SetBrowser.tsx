import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import SortIcon from "./icons/SortIcon.tsx";
import CheckIcon from "./icons/CheckIcon.tsx";
import { statuses } from "../lib/status.ts";
import SelectPicker from "./SelectPicker.tsx";
import SetSearch from "./SetSearch.tsx";
import LanguagePicker from "./LanguagePicker.tsx";
import { useHydrated } from "../hooks/useHydrated.ts";
import { useRef, useState } from "react";
import { filterSets, languageName, type Group } from "../lib/model.ts";

import SetCard from "./SetCard.tsx";
export default function SetBrowser({ groups, revision }: { groups: Group[]; revision: string }) {
  const [query, setQuery] = useState("");
  const [language, setLanguage] = useState("");
  const [status, setStatus] = useState("");
  const [sort, setSort] = useState("default");
  const [size, setSize] = useState(25);
  const [page, setPage] = useState(0);
  const ready = useHydrated();
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const languages = [
    ...new Set(groups.flatMap((group) => group.releases.map((row) => row.language))),
  ].sort((a, b) => languageName(a).localeCompare(languageName(b), "en"));
  const filtered = filterSets(groups, query, language, status, sort);
  const start = page * size;
  const visible = ready ? filtered.slice(start, start + size) : filtered;
  function changePage(offset: number) {
    setPage(page + offset);
    // Focus the first visible summary after React commits the new page.
    requestAnimationFrame(() => list.current?.querySelector("summary")?.focus());
  }
  return (
    <section data-ui="sets" {...stylex.props(styles.sets)} aria-labelledby="sets-title">
      <h2 id="sets-title" data-ui="sr-only" {...stylex.props(styles.srOnly)}>
        Set coverage
      </h2>
      <form
        id="filters"
        data-ui="filters"
        {...stylex.props(styles.filters)}
        role="search"
        inert={!ready}
        onSubmit={(event) => event.preventDefault()}
        onChange={() => setPage(0)}
        onReset={(event) => {
          event.preventDefault();
          setQuery("");
          setLanguage("");
          setStatus("");
          setSort("default");
          setPage(0);
          input.current?.focus();
        }}
      >
        <SetSearch
          groups={groups}
          value={query}
          inputRef={input}
          onChange={(value) => {
            setQuery(value);
            setPage(0);
          }}
        />
        <LanguagePicker
          languages={languages}
          value={language}
          onChange={(value) => {
            setLanguage(value);
            setPage(0);
          }}
        />
        <SelectPicker
          label="Status"
          value={status}
          onChange={(value) => {
            setStatus(value);
            setPage(0);
          }}
          options={[
            { value: "", label: "Status", icon: <CheckIcon /> },
            ...Object.entries(statuses).map(([value, presentation]) => ({
              value,
              label: presentation.label,
              icon: <span {...stylex.props(styles.dot, presentation.dot)} aria-hidden="true" />,
            })),
            { value: "finishes", label: "Missing finishes", icon: <CheckIcon /> },
            { value: "illustrators", label: "Missing illustrators", icon: <CheckIcon /> },
          ]}
        />
        <SelectPicker
          label="Sort by"
          value={sort}
          onChange={(value) => {
            setSort(value);
            setPage(0);
          }}
          options={[
            { value: "default", label: "Sort By" },
            { value: "name", label: "Set name" },
            { value: "cards", label: "Most printings" },
            { value: "gaps", label: "Most missing finishes" },
          ].map((option) => ({ ...option, icon: <SortIcon /> }))}
        />
        <button
          {...stylex.props(styles.button)}
          type="reset"
          id="clear"
          hidden={!query.trim() && !language && !status && sort === "default"}
        >
          Clear filters
        </button>
      </form>
      <p id="results" data-ui="sr-only" {...stylex.props(styles.srOnly)} role="status">
        {filtered.length} {filtered.length === 1 ? "set" : "sets"} found
      </p>
      <div {...stylex.props(styles.setList)} id="set-list" ref={list}>
        {visible.map((group) => (
          <SetCard key={group.releases[0].setId} group={group} revision={revision} />
        ))}
      </div>
      <p id="empty" hidden={filtered.length > 0}>
        {groups.length ? "No sets match these filters." : "No sets are recorded yet."}
      </p>
      <div
        data-ui="pagination"
        {...stylex.props(styles.pagination)}
        id="pagination"
        hidden={!ready || filtered.length <= 25}
      >
        <div data-ui="page-size" {...stylex.props(styles.pageSize)}>
          <span>Sets per page</span>
          <SelectPicker
            label="Sets per page"
            value={String(size)}
            onChange={(value) => {
              setSize(Number(value));
              setPage(0);
            }}
            options={[25, 50, 100].map((value) => ({ value: String(value), label: String(value) }))}
          />
        </div>
        <span id="page-status">
          {filtered.length ? start + 1 : 0}–{Math.min(start + size, filtered.length)} of{" "}
          {filtered.length}
        </span>
        <div>
          <button
            {...stylex.props(styles.button)}
            type="button"
            disabled={page === 0}
            onClick={() => changePage(-1)}
          >
            Previous
          </button>{" "}
          <button
            {...stylex.props(styles.button)}
            type="button"
            disabled={start + size >= filtered.length}
            onClick={() => changePage(1)}
          >
            Next
          </button>
        </div>
      </div>
    </section>
  );
}
