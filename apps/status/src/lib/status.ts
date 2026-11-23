import { styles } from "../styles.stylex.ts";
export const statuses = {
  empty: { label: "Not Started", dot: styles.redDot, bar: styles.redBar, badge: styles.redBadge },
  partial: {
    label: "In Progress",
    dot: styles.orangeDot,
    bar: styles.orangeBar,
    badge: styles.orangeBadge,
  },
  done: { label: "Done", dot: styles.greenDot, bar: styles.greenBar, badge: styles.greenBadge },
};
