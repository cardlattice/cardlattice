import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import { statuses } from "../lib/status.ts";
import { type Group } from "../lib/model.ts";
export default function Progress({ groups }: { groups: Group[] }) {
  const progress = Object.entries(statuses).map(([status, presentation]) => ({
    presentation,
    status,
    count: groups.filter((group) => group.status === status).length,
  }));
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
      <svg
        data-ui="progress-bar"
        {...stylex.props(styles.bar)}
        width="100%"
        height="14"
        viewBox="0 0 100 14"
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        {progress.map(({ status, count, presentation }, index) => (
          <rect
            key={status}
            className={stylex.props(presentation.bar).className}
            x={
              groups.length
                ? (progress.slice(index + 1).reduce((sum, item) => sum + item.count, 0) /
                    groups.length) *
                  100
                : 0
            }
            width={groups.length ? (count / groups.length) * 100 : 0}
            height="14"
          />
        ))}
      </svg>
    </section>
  );
}
