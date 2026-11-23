import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
import ChevronIcon from "./icons/ChevronIcon.tsx";
import { Autocomplete } from "@base-ui/react/autocomplete";
import { useState, type Ref } from "react";
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
  const [open, setOpen] = useState(false);
  const suggestions = [
    ...new Set(filterSets(groups, value, "", "", "default").map((group) => group.releases[0].name)),
  ].slice(0, 10);
  return (
    <Autocomplete.Root
      items={suggestions}
      filter={null}
      value={value}
      onValueChange={onChange}
      open={open}
      onOpenChange={setOpen}
      openOnInputClick
    >
      <div data-ui="set-search" {...stylex.props(styles.search)}>
        <Autocomplete.Input
          {...stylex.props(styles.searchInput)}
          ref={inputRef}
          onFocus={() => setOpen(true)}
          id="query"
          aria-label="Find a set, game, series or ID"
          placeholder="Find a set, game, series or ID"
        />
        <Autocomplete.Trigger
          onMouseDown={(event) => {
            if (open) {
              event.preventDefault();
              event.preventBaseUIHandler();
              setOpen(false);
            }
          }}
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

const styles = stylex.create({
  search: {
    position: "relative",
    width: { default: "min(100%, 305px)", "@media (max-width: 640px)": "100%" },
  },
  searchInput: {
    width: "100%",
    height: 34,
    backgroundColor: "transparent",
    borderWidth: 1,
    borderStyle: "solid",
    borderColor: colors.borderInteractive,
    borderRadius: 8,
    paddingLeft: 7,
    paddingRight: 32,
    paddingBlock: 0,
    color: colors.primary,
    "::placeholder": { color: colors.secondary, opacity: 1 },
  },
  searchTrigger: {
    position: "absolute",
    top: 1,
    right: 1,
    bottom: 1,
    display: "grid",
    placeItems: "center",
    width: 30,
    padding: 0,
    borderWidth: 0,
    borderRadius: 7,
    backgroundColor: { default: "transparent", ":hover": colors.hover },
    color: { default: colors.secondary, ":hover": colors.primary },
  },
  positioner: { zIndex: 20 },
  popup: {
    minWidth: "max(var(--anchor-width), 220px)",
    maxWidth: "min(400px, calc(100vw - 24px))",
    maxHeight: "min(320px, var(--available-height))",
    overflowY: "auto",
    overscrollBehavior: "contain",
    padding: 5,
    borderWidth: 1,
    borderStyle: "solid",
    borderColor: colors.borderInteractive,
    borderRadius: 10,
    backgroundColor: colors.canvas,
    color: colors.primary,
    boxShadow: "0 8px 24px rgb(0 0 0 / 12%)",
    fontSize: 13,
  },
  pickerEmpty: {
    paddingBlock: 12,
    paddingInline: 10,
    color: colors.secondary,
    lineHeight: 1.5,
    ":empty": { display: "none" },
  },
  option: {
    display: "flex",
    alignItems: "center",
    gap: 10,
    minHeight: 38,
    paddingBlock: 8,
    paddingInline: 10,
    borderRadius: 6,
    cursor: "pointer",
    outlineStyle: "none",
  },
  highlighted: {
    backgroundColor: colors.hover,
    outlineWidth: 1,
    outlineStyle: "solid",
    outlineColor: colors.borderInteractive,
    outlineOffset: -1,
  },
});
