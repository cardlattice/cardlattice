import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
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
