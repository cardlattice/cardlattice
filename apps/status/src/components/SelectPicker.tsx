import { Select } from "@base-ui/react/select";
import type { ReactNode } from "react";
import Icon from "./Icon.tsx";

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
  const selected = options.find((option) => option.value === value)!;
  return (
    <Select.Root value={value} onValueChange={(next) => onChange(next || "")} modal={false}>
      <Select.Trigger className="select-trigger" aria-label={label}>
        {selected.icon}
        <span>{triggerLabel ?? selected.label}</span>
        <Select.Icon>
          <Icon name="chevron" />
        </Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Positioner
          sideOffset={6}
          align="start"
          alignItemWithTrigger={false}
          className="picker-positioner"
        >
          <Select.Popup className="picker-popup">
            <Select.List aria-label={label}>
              {options.map((option) => (
                <Select.Item key={option.value} value={option.value} className="picker-option">
                  {option.icon}
                  <Select.ItemText>{option.label}</Select.ItemText>
                  <Select.ItemIndicator className="picker-check">
                    <Icon name="check" />
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
