import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import ChevronIcon from "./icons/ChevronIcon.tsx";
import { Autocomplete } from "@base-ui/react/autocomplete";
import type { Ref } from "react";
import { filterSets, type Group } from "../lib/model.ts";

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
      <div data-ui="set-search" {...stylex.props(styles.search)}>
        <Autocomplete.Input
          {...stylex.props(styles.searchInput)}
          ref={inputRef}
          id="query"
          aria-label="Find a set, game, series or ID"
          placeholder="Find a set, game, series or ID"
        />
        <Autocomplete.Trigger
          data-ui="search-trigger"
          {...stylex.props(styles.searchTrigger)}
          aria-label="Show set suggestions"
        >
          <ChevronIcon />
        </Autocomplete.Trigger>
      </div>
      <Autocomplete.Portal>
        <Autocomplete.Positioner
          sideOffset={6}
          align="start"
          data-ui="picker-positioner"
          {...stylex.props(styles.positioner)}
        >
          <Autocomplete.Popup data-ui="picker-popup" {...stylex.props(styles.popup)}>
            <Autocomplete.Empty data-ui="picker-empty" {...stylex.props(styles.pickerEmpty)}>
              No matching sets. Try a different name, game, series or ID.
            </Autocomplete.Empty>
            <Autocomplete.List aria-label="Set suggestions">
              {(item: string) => (
                <Autocomplete.Item
                  key={item}
                  value={item}
                  data-ui="picker-option"
                  className={(state) =>
                    stylex.props(styles.option, state.highlighted && styles.highlighted).className
                  }
                >
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
