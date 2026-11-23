import type { CSSProperties } from "react";
import { statusLabels, type Group } from "../lib/model.ts";
export default function Progress({ groups }: { groups: Group[] }) {
  const progress = (["empty", "partial", "done"] as const).map((status) => ({
    status,
    count: groups.filter((group) => group.status === status).length,
  }));
  const format = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 1 });
  return (
    <section className="progress" aria-labelledby="progress-title">
      <div className="progress-heading">
        <h2 id="progress-title">Progress</h2>
        <ul className="progress-legend">
          {progress.map(({ status, count }) => (
            <li key={status} className={status}>
              {format.format(groups.length ? count / groups.length : 0)}{" "}
              {statusLabels[status].toLowerCase()}
            </li>
          ))}
        </ul>
      </div>
      <div className="progress-bar" aria-hidden="true">
        {[...progress]
          .reverse()
          .filter(({ count }) => count > 0)
          .map(({ status, count }) => (
            <span key={status} className={status} style={{ "--count": count } as CSSProperties} />
          ))}
      </div>
    </section>
  );
}
