import { useState, type FormEvent } from "react";
import { request, authorized } from "../api";
import { PageHeader } from "../components/PageHeader";
import { Field } from "../components/Field";
import { Button } from "../components/Button";
import { ReviewCard, type PendingContribution } from "../components/ReviewCard";

type Outbox = {
  id: string;
  status: string;
  attempts: number;
  last_error: string | null;
  revision: number;
};
type ReviewState = {
  queue: PendingContribution[] | null;
  outbox: Outbox[];
  cursor: string | null;
  error: string;
  pending: boolean;
};
const initialReviewState: ReviewState = {
  queue: null,
  outbox: [],
  cursor: null,
  error: "",
  pending: false,
};
export function Moderate() {
  const [token, setToken] = useState("");
  const [{ queue, outbox, cursor, error, pending }, setReviewState] = useState(initialReviewState);
  async function load(next?: string) {
    setReviewState((state) => ({ ...state, pending: true, error: "" }));
    try {
      const response = await request<{ data: PendingContribution[]; nextCursor: string | null }>(
        `/v1/moderation/contributions${next ? `?cursor=${next}` : ""}`,
        authorized(token),
      );
      const publication = await request<{ data: Outbox[] }>(
        "/v1/moderation/outbox",
        authorized(token),
      );
      setReviewState({
        queue: response.data,
        cursor: response.nextCursor,
        outbox: publication.data,
        error: "",
        pending: false,
      });
    } catch (cause) {
      setReviewState({
        ...initialReviewState,
        error: cause instanceof Error ? cause.message : "Could not load review queue",
      });
    }
  }
  async function replay(id: string) {
    try {
      await request(`/v1/moderation/outbox/${id}/replay`, { ...authorized(token), method: "POST" });
      await load();
    } catch (cause) {
      setReviewState((state) => ({
        ...state,
        error: cause instanceof Error ? cause.message : "Replay failed",
      }));
    }
  }
  function submit(event: FormEvent) {
    event.preventDefault();
    void load();
  }
  return (
    <div className="page narrow">
      <PageHeader eyebrow="CARE BEFORE COVERAGE" title="The review desk.">
        <p>
          Check the metadata, its source, and its relationships. Every decision leaves an audit
          trail.
        </p>
      </PageHeader>
      <form onSubmit={submit} className="panel">
        <Field
          label="Moderator token"
          hint="Local development: docker compose exec api cat /run/cardlattice/moderator. Kept in memory for this visit only."
        >
          {(id, hint) => (
            <input
              id={id}
              aria-labelledby={`${id}-label`}
              type="password"
              aria-describedby={hint}
              value={token}
              disabled={pending}
              onChange={(event) => {
                setToken(event.target.value);
                setReviewState(initialReviewState);
              }}
              required
              autoComplete="off"
              minLength={32}
            />
          )}
        </Field>
        <Button type="submit" disabled={pending}>
          {pending ? "Loading…" : "Load review queue"}
        </Button>
      </form>
      {error && (
        <p className="notice error" role="alert">
          {error}
        </p>
      )}
      {queue && (
        <section className="section">
          <div className="section-heading">
            <h2>Pending contributions</h2>
            <span className="badge">{queue.length} on this page</span>
          </div>
          {queue.length ? (
            queue.map((contribution) => (
              <ReviewCard
                key={contribution.id}
                contribution={contribution}
                token={token}
                onReviewed={() => void load()}
              />
            ))
          ) : (
            <div className="empty">
              <h3>The queue is clear.</h3>
              <p>No pending contributions on this page.</p>
            </div>
          )}
          {cursor && <Button onClick={() => void load(cursor)}>Next page →</Button>}
        </section>
      )}
      {queue && (
        <section className="section">
          <h2>Publication & cache invalidation</h2>
          <p>Latest 100 events. Failed events can be replayed after resolving the cause.</p>
          <ul className="outbox-list">
            {outbox.map((event) => (
              <li key={event.id}>
                <div>
                  <strong>Revision {event.revision}</strong>
                  <p>
                    {event.status} · {event.attempts} attempts
                  </p>
                  {event.last_error && <p className="field-error">{event.last_error}</p>}
                </div>
                {event.status === "failed" && (
                  <Button onClick={() => void replay(event.id)}>Replay</Button>
                )}
              </li>
            ))}
          </ul>
          <Button onClick={() => void load()}>Refresh publication state</Button>
        </section>
      )}
    </div>
  );
}
