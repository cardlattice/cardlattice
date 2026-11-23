import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import { percent } from "../lib/coverage-model.ts";
import { n } from "../lib/model.ts";
export default function Recorded({
  value,
  cards,
  ring = true,
}: {
  value: number;
  cards: number;
  ring?: boolean;
}) {
  return (
    <span data-ui="recorded" {...stylex.props(styles.recorded)}>
      {ring && (
        <svg
          data-ui="coverage-ring"
          data-complete={value === cards && cards > 0}
          {...stylex.props(styles.ring)}
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <circle cx="6" cy="6" r="5" />
          <circle
            data-ui="coverage-value"
            {...stylex.props(styles.ringValue, value === cards && cards > 0 && styles.ringComplete)}
            cx="6"
            cy="6"
            r="5"
            pathLength="100"
            strokeDasharray={`${percent(value, cards)} 100`}
            transform="rotate(-90 6 6)"
          />
        </svg>
      )}
      <span>
        {n(value)}
        <span data-ui="secondary" {...stylex.props(styles.secondary)}>
          /{n(cards)}
        </span>
      </span>
    </span>
  );
}
