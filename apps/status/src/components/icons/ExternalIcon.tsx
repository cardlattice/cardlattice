import * as stylex from "@stylexjs/stylex";
import { styles } from "../../styles.stylex.ts";
export default function ExternalIcon() {
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
      <path d="M8.5 3H4a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h7a2 2 0 0 0 2-2V8m-3-6h4v4m0-4L7 9" />
    </svg>
  );
}
