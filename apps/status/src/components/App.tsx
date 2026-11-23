import { colors } from "../palette.stylex.ts";
import * as stylex from "@stylexjs/stylex";
import type { Coverage } from "../lib/coverage-model.ts";
import { groupSets, n, releaseLanguages } from "../lib/model.ts";
import Header from "./Header.tsx";
import Progress from "./Progress.tsx";
import SetBrowser from "./SetBrowser.tsx";
import Footer from "./Footer.tsx";
export default function App({ rows, revision }: { rows: Coverage[]; revision: string }) {
  const groups = groupSets(rows);
  return (
    <>
      <a data-ui="skip" {...stylex.props(styles.skip)} href="#main">
        Skip to content
      </a>
      <Header revision={revision} />
      <main id="main" data-ui="container" {...stylex.props(styles.container)}>
        <section data-ui="intro" {...stylex.props(styles.intro)} aria-labelledby="page-title">
          <h1 {...stylex.props(styles.title)} id="page-title">
            Catalogue Progress
          </h1>
          <p {...stylex.props(styles.subtitle)}>
            Explore what’s recorded, find what’s missing, and help make open card data better.
          </p>
        </section>
        <dl data-ui="stats" {...stylex.props(styles.stats)} aria-label="Catalogue totals">
          <div {...stylex.props(styles.stat)}>
            <dt {...stylex.props(styles.statLabel)}>Known Printings</dt>
            <dd {...stylex.props(styles.statValue)}>
              {n(rows.reduce((sum, row) => sum + row.cards, 0))}
            </dd>
          </div>
          <div {...stylex.props(styles.stat)}>
            <dt {...stylex.props(styles.statLabel)}>Sets in the repository</dt>
            <dd {...stylex.props(styles.statValue)}>{n(groups.length)}</dd>
          </div>
          <div {...stylex.props(styles.stat)}>
            <dt {...stylex.props(styles.statLabel)}>Languages with cards</dt>
            <dd {...stylex.props(styles.statValue)}>
              {n(releaseLanguages(rows.filter((row) => row.cards > 0)).size)}
            </dd>
          </div>
        </dl>
        <Progress groups={groups} />
        <SetBrowser groups={groups} revision={revision} />
      </main>
      <Footer />
    </>
  );
}

const styles = stylex.create({
  skip: {
    position: "absolute",
    top: 8,
    left: 8,
    zIndex: 1,
    transform: { default: "translateY(-150%)", ":focus": "none" },
    padding: 8,
    backgroundColor: colors.canvas,
  },
  container: {
    width: {
      default: "min(870px, calc(100% - 32px))",
      "@media (max-width: 390px)": "calc(100% - 24px)",
    },
    marginInline: "auto",
  },
  intro: { marginBottom: 30 },
  title: { margin: 0, fontSize: 32, lineHeight: "40px", fontWeight: 500, letterSpacing: "-0.01em" },
  subtitle: { marginTop: 13, marginBottom: 0, color: colors.secondary, letterSpacing: "-0.013em" },
  stats: {
    display: "grid",
    gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
    margin: 0,
    paddingBlock: 18,
    borderBlockWidth: 1,
    borderBlockStyle: "solid",
    borderBlockColor: colors.border,
    gap: { default: 0, "@media (max-width: 560px)": 10 },
  },
  stat: { display: "flex", flexDirection: "column", justifyContent: "space-between" },
  statLabel: {
    color: colors.secondary,
    fontSize: { default: 14, "@media (max-width: 560px)": 12 },
    lineHeight: "20px",
    fontWeight: 400,
  },
  statValue: {
    marginTop: 12,
    marginInline: 0,
    marginBottom: 0,
    fontSize: 29,
    fontWeight: 500,
    lineHeight: "38px",
  },
});
