import { Tooltip } from "@base-ui/react/tooltip";
import * as stylex from "@stylexjs/stylex";
import { colors } from "../palette.stylex.ts";
import { n } from "../lib/model.ts";

export default function ProgressSegment({
  status,
  label,
  count,
  total,
  x,
}: {
  status: string;
  label: string;
  count: number;
  total: number;
  x: number;
}) {
  const percentage = new Intl.NumberFormat("en-GB", {
    style: "percent",
    maximumFractionDigits: 1,
  }).format(count / total);
  const description = `${label}: ${n(count)} of ${n(total)} ${total === 1 ? "set" : "sets"} (${percentage})`;
  return (
    <Tooltip.Root>
      <Tooltip.Trigger
        render={<rect x={x} width={(count / total) * 100} height="14" />}
        role="img"
        tabIndex={0}
        aria-label={description}
        {...stylex.props(styles.segment, fills.get(status))}
      />
      <Tooltip.Portal>
        <Tooltip.Positioner side="top" sideOffset={8} {...stylex.props(styles.positioner)}>
          <Tooltip.Popup role="tooltip" {...stylex.props(styles.popup)}>
            {description}
          </Tooltip.Popup>
        </Tooltip.Positioner>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

const styles = stylex.create({
  segment: {
    outlineStyle: "none",
    stroke: { default: "none", ":focus-visible": colors.focus },
    strokeWidth: 1,
    vectorEffect: "non-scaling-stroke",
  },
  empty: { fill: colors.redBar },
  partial: { fill: colors.orangeBar },
  done: { fill: colors.greenBar },
  positioner: { zIndex: 30 },
  popup: {
    maxWidth: "calc(100vw - 24px)",
    paddingBlock: 8,
    paddingInline: 10,
    borderWidth: 1,
    borderStyle: "solid",
    borderColor: colors.borderInteractive,
    borderRadius: 8,
    backgroundColor: colors.canvas,
    color: colors.primary,
    boxShadow: "0 4px 16px rgb(0 0 0 / 15%)",
    fontSize: 13,
  },
});
const fills = new Map([
  ["empty", styles.empty],
  ["partial", styles.partial],
  ["done", styles.done],
]);
