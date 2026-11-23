import { Autocomplete } from "@base-ui/react/autocomplete";
import type { Ref } from "react";
import { filterSets, type Group } from "../lib/model.ts";
import Icon from "./Icon.tsx";

export default function SetSearch({
  groups,
  value,
  onChange,
  inputRef,
}: {
  groups: Group[];
  value: string;
  onChange: (value: string) => void;
  inputRef: Ref<HTMLInputElement>;
}) {
  const suggestions = [
    ...new Set(filterSets(groups, value, "", "", "default").map((group) => group.releases[0].name)),
  ].slice(0, 10);
  return (
    <Autocomplete.Root items={suggestions} filter={null} value={value} onValueChange={onChange}>
      <div className="set-search">
        <Autocomplete.Input
          ref={inputRef}
          id="query"
          aria-label="Find a set, game, series or ID"
          placeholder="Find a set, game, series or ID"
        />
        <Autocomplete.Trigger className="search-trigger" aria-label="Show set suggestions">
          <Icon name="chevron" />
        </Autocomplete.Trigger>
      </div>
      <Autocomplete.Portal>
        <Autocomplete.Positioner sideOffset={6} align="start" className="picker-positioner">
          <Autocomplete.Popup className="picker-popup">
            <Autocomplete.Empty className="picker-empty">
              No matching sets. Try a different name, game, series or ID.
            </Autocomplete.Empty>
            <Autocomplete.List aria-label="Set suggestions">
              {(item: string) => (
                <Autocomplete.Item key={item} value={item} className="picker-option">
                  <span>{item}</span>
                </Autocomplete.Item>
              )}
            </Autocomplete.List>
          </Autocomplete.Popup>
        </Autocomplete.Positioner>
      </Autocomplete.Portal>
    </Autocomplete.Root>
  );
}
