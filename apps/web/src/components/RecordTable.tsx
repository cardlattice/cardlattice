import { Link } from "@tanstack/react-router";
import { ArrowUpRight, Layers3 } from "lucide-react";
import type { RecordData } from "../api";

function recordPath(record: RecordData, game?: string) {
  if (record.kind === "game") return `/games/${record.slug}`;
  const collection = record.kind === "series" ? "series" : `${record.kind}s`;
  return `/games/${game}/${collection}/${record.id}`;
}
export function RecordTable({ records, game }: { records: RecordData[]; game?: string }) {
  if (!records.length)
    return (
      <div className="empty">
        <Layers3 aria-hidden="true" size={32} />
        <h2>No records here yet</h2>
        <p>
          This catalogue grows through reviewed, original contributions. Try another filter or
          contribute a correction.
        </p>
        <Link to="/contribute" className="text-link">
          Contribute metadata <ArrowUpRight size={16} aria-hidden="true" />
        </Link>
      </div>
    );
  return (
    <ul className="record-list">
      {records.map((record) => (
        <li key={record.id}>
          <Link to={recordPath(record, game || record.gameId || "pokemon")} className="record-row">
            <span className="record-symbol" aria-hidden="true">
              <Layers3 size={22} />
            </span>
            <span className="record-main">
              <strong>{record.name}</strong>
              <span>
                {record.collectorNumber || record.kind}{" "}
                {record.language && ` · ${record.language.toUpperCase()}`}{" "}
                {record.rarity && ` · ${record.rarity}`}
              </span>
            </span>
            {record.synthetic && <span className="badge">Synthetic</span>}
            <ArrowUpRight className="row-arrow" size={18} aria-hidden="true" />
          </Link>
        </li>
      ))}
    </ul>
  );
}
