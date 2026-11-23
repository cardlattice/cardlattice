import { Tooltip } from "@base-ui/react/tooltip";
import ProgressSegment from "./ProgressSegment.tsx";
import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
import { statusLabels, type Group } from "../lib/model.ts";
export default function Progress({ groups }: { groups: Group[] }) {
  const progress = Object.entries(statuses).map(([status, presentation]) => ({
    presentation,
    status,
    count: groups.filter((group) => group.status === status).length,
  }));
  const segments = progress.filter(({ count }) => count > 0);
  const format = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 1 });
  return (
    <section data-ui="progress" {...stylex.props(styles.progress)} aria-labelledby="progress-title">
      <div data-ui="progress-heading" {...stylex.props(styles.progressHeading)}>
        <h2 {...stylex.props(styles.progressTitle)} id="progress-title">
          Progress
        </h2>
        <ul data-ui="progress-legend" {...stylex.props(styles.legend)}>
          {progress.map(({ status, count, presentation }) => (
            <li key={status} {...stylex.props(styles.legendItem)}>
              <span aria-hidden="true" {...stylex.props(styles.dot, presentation.dot)} />
              {format.format(groups.length ? count / groups.length : 0)}{" "}
              {presentation.label.toLowerCase()}
            </li>
          ))}
        </ul>
      </div>
      <Tooltip.Provider delay={150}>
        <svg
          data-ui="progress-bar"
          {...stylex.props(styles.bar)}
          width="100%"
          height="14"
          viewBox="0 0 100 14"
          preserveAspectRatio="none"
          role="group"
          aria-label="Set status distribution"
        >
          {segments.map(({ status, count, presentation }, index) => (
            <ProgressSegment
              key={status}
              status={status}
              label={presentation.label}
              count={count}
              total={groups.length}
              x={
                (segments.slice(index + 1).reduce((sum, item) => sum + item.count, 0) /
                  groups.length) *
                100
              }
            />
          ))}
        </svg>
      </Tooltip.Provider>
    </section>
  );
}

const styles = stylex.create({
  progress: {
    paddingTop: 19,
    paddingBottom: 20,
    borderBottomWidth: 1,
    borderBottomStyle: "solid",
    borderBottomColor: colors.border,
  },
  progressHeading: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    rowGap: 8,
    columnGap: 16,
  },
  progressTitle: {
    margin: 0,
    color: colors.secondary,
    fontSize: 14,
    lineHeight: "20px",
    fontWeight: 400,
  },
  legend: {
    display: "flex",
    flexWrap: "wrap",
    rowGap: 6,
    columnGap: 20,
    padding: 0,
    margin: 0,
    color: colors.secondary,
    fontSize: 13,
    listStyleType: "none",
  },
  legendItem: { display: "inline-flex", alignItems: "center", gap: 6, whiteSpace: "nowrap" },
  dot: { display: "inline-block", width: 8, height: 8, borderRadius: "50%", flexShrink: 0 },
  bar: {
    display: "flex",
    overflow: "hidden",
    height: 14,
    marginTop: 14,
    borderRadius: 7,
    backgroundColor: colors.hover,
  },
  redDot: { backgroundColor: colors.red },
  orangeDot: { backgroundColor: colors.orange },
  greenDot: { backgroundColor: colors.green },
});

const statuses = {
  empty: { label: statusLabels.empty, dot: styles.redDot },
  partial: { label: statusLabels.partial, dot: styles.orangeDot },
  done: { label: statusLabels.done, dot: styles.greenDot },
};
