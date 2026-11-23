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
      <a className="skip" href="#main">
        Skip to content
      </a>
      <Header revision={revision} />
      <main id="main" className="container">
        <section className="intro" aria-labelledby="page-title">
          <h1 id="page-title">Catalogue Progress</h1>
          <p>Explore what’s recorded, find what’s missing, and help make open card data better.</p>
        </section>
        <dl className="stats" aria-label="Catalogue totals">
          <div>
            <dt>Known Printings</dt>
            <dd>{n(rows.reduce((sum, row) => sum + row.cards, 0))}</dd>
          </div>
          <div>
            <dt>Sets in the repository</dt>
            <dd>{n(groups.length)}</dd>
          </div>
          <div>
            <dt>Languages with cards</dt>
            <dd>{n(releaseLanguages(rows.filter((row) => row.cards > 0)).size)}</dd>
          </div>
        </dl>
        <Progress groups={groups} />
        <SetBrowser groups={groups} revision={revision} />
      </main>
      <Footer />
    </>
  );
}
