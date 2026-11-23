import * as stylex from "@stylexjs/stylex";
export default function SortIcon() {
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
      <path d="M4.5 2.5v11m-3-3 3 3 3-3m4-8v11m-3-8 3-3 3 3" />
    </svg>
  );
}

const styles = stylex.create({
  icon: { display: "block", flexShrink: 0, width: 16, height: 16 },
});
