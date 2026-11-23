import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
import CheckIcon from "./icons/CheckIcon.tsx";
import ChevronIcon from "./icons/ChevronIcon.tsx";
import { Select } from "@base-ui/react/select";
import type { ReactNode } from "react";

export interface SelectOption {
  value: string;
  label: string;
  icon?: ReactNode;
}
export default function SelectPicker({
  label,
  value,
  onChange,
  options,
  triggerLabel,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  triggerLabel?: string;
}) {
  const selected = options.find((option) => option.value === value);
  return (
    <Select.Root value={value} onValueChange={(next) => onChange(next || "")} modal={false}>
      <Select.Trigger
        data-ui="select-trigger"
        {...stylex.props(styles.selectTrigger)}
        aria-label={label}
      >
        {selected?.icon}
        <span>{triggerLabel ?? selected?.label ?? label}</span>
        <Select.Icon>
          <ChevronIcon />
        </Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Positioner
          sideOffset={6}
          align="start"
          alignItemWithTrigger={false}
          data-ui="picker-positioner"
          {...stylex.props(styles.positioner)}
        >
          <Select.Popup data-ui="picker-popup" {...stylex.props(styles.popup)}>
            <Select.List aria-label={label}>
              {options.map((option) => (
                <Select.Item
                  key={option.value}
                  value={option.value}
                  data-ui="picker-option"
                  className={(state) =>
                    stylex.props(styles.option, state.highlighted && styles.highlighted).className
                  }
                >
                  {option.icon}
                  <Select.ItemText>{option.label}</Select.ItemText>
                  <Select.ItemIndicator
                    data-ui="picker-check"
                    {...stylex.props(styles.pickerCheck)}
                  >
                    <CheckIcon />
                  </Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.List>
          </Select.Popup>
        </Select.Positioner>
      </Select.Portal>
    </Select.Root>
  );
}

const styles = stylex.create({
  selectTrigger: {
    display: "inline-flex",
    alignItems: "center",
    gap: 8,
    minHeight: 34,
    paddingBlock: 0,
    paddingInline: 6,
    borderWidth: 0,
    borderRadius: 6,
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
  pickerCheck: { display: "flex", marginLeft: "auto" },
});
