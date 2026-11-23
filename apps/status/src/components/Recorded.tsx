import { percent } from "../lib/coverage-model.ts";
import { n } from "../lib/model.ts";
export default function Recorded({
  value,
  cards,
  ring = true,
}: {
  value: number;
  cards: number;
  ring?: boolean;
}) {
  return (
    <span className="recorded">
      {ring && (
        <svg
          className={`coverage-ring ${value === cards && cards > 0 ? "complete" : ""}`}
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <circle cx="6" cy="6" r="5" />
          <circle
            className="coverage-value"
            cx="6"
            cy="6"
            r="5"
            pathLength="100"
            strokeDasharray={`${percent(value, cards)} 100`}
            transform="rotate(-90 6 6)"
          />
        </svg>
      )}
      <span>
        {n(value)}
        <span className="secondary">/{n(cards)}</span>
      </span>
    </span>
  );
}
