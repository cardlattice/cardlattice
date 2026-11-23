import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
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
