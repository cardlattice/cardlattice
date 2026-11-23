import * as stylex from "@stylexjs/stylex";
import { styles } from "../../styles.stylex.ts";
export default function CheckIcon() {
  return (
    <svg
      {...stylex.props(styles.icon)}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.25"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="m2.5 8 3.5 3.5 7.5-7.5" />
    </svg>
  );
}
