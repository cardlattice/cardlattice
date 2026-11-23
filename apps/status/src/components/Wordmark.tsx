import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
export default function Wordmark() {
  return (
    <>
      <span {...stylex.props(styles.mark)} aria-hidden="true">
        <svg
          {...stylex.props(styles.markIcon)}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="m2.5 7.25 9.5-4.75 9.5 4.75L12 12 2.5 7.25Zm0 5L12 17l9.5-4.75m-19 5L12 22l9.5-4.75" />
        </svg>
      </span>
      <span>
        cardlattice<span {...stylex.props(styles.period)}>.</span>
      </span>
    </>
  );
}
