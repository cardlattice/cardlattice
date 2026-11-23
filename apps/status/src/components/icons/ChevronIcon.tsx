import * as stylex from "@stylexjs/stylex";
export default function ChevronIcon() {
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
      <path d="m4 6 4 4 4-4" />
    </svg>
  );
}

const styles = stylex.create({
  icon: { display: "block", flexShrink: 0, width: 16, height: 16 },
});
