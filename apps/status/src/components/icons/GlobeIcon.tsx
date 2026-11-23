import * as stylex from "@stylexjs/stylex";
export default function GlobeIcon() {
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
      <circle cx="8" cy="8" r="6.5" />
      <ellipse cx="8" cy="8" rx="2.8" ry="6.5" />
      <path d="M1.5 8h13" />
    </svg>
  );
}

const styles = stylex.create({
  icon: { display: "block", flexShrink: 0, width: 16, height: 16 },
});
