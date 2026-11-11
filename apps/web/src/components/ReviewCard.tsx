import { useState } from "react";
import { request, authorized } from "../api";
import { Button } from "./Button";
import { Field } from "./Field";

export type PendingContribution = {
  id: string;
  operation: string;
  payload: { name: string; kind: string; source?: { description: string; license: string } | null };
  createdAt: string;
};
export function ReviewCard({
  contribution,
  token,
  onReviewed,
}: {
  contribution: PendingContribution;
  token: string;
  onReviewed: () => void;
}) {
  const [reason, setReason] = useState("");
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState("");
  const [scans, setScans] = useState<
    { id: string; side: string; mediaType: string; rightsStatus: string }[] | null
  >(null);
  async function review(decision: string) {
    setPending(true);
    try {
      const result = await request<{ revision: number | null }>(
        `/v1/moderation/contributions/${contribution.id}/review`,
        authorized(token, { decision, reason }),
      );
      setMessage(
        decision === "approve"
          ? `Approved at revision ${result.revision}. Publication and cache invalidation are queued.`
          : "Rejected. The decision is recorded in the audit history.",
      );
      onReviewed();
    } catch (cause) {
      setMessage(cause instanceof Error ? cause.message : "Review failed");
    } finally {
      // Both decision buttons are disabled while this single review is pending.
      setPending(false);
    }
  }
  async function loadScans() {
    try {
      const response = await request<{ scans: NonNullable<typeof scans> }>(
        `/v1/contributions/${contribution.id}`,
        authorized(token),
      );
      setScans(response.scans);
    } catch (cause) {
      setMessage(cause instanceof Error ? cause.message : "Could not load scans");
    }
  }
  async function download(id: string, side: string) {
    try {
      const response = await fetch(`/v1/moderation/scans/${id}`, authorized(token));
      if (!response.ok) throw new Error("Scan download failed");
      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${id}-${side}.${blob.type === "image/png" ? "png" : "jpg"}`;
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (cause) {
      setMessage(cause instanceof Error ? cause.message : "Download failed");
    }
  }
  return (
    <article className="panel review-card">
      <p className="eyebrow">
        {contribution.operation} · {contribution.payload.kind}
      </p>
      <h2>{contribution.payload.name}</h2>
      {contribution.payload.source && <p>{contribution.payload.source.description}</p>}
      <details>
        <summary>Inspect complete draft & source</summary>
        <pre className="code-block">
          <code>{JSON.stringify(contribution.payload, null, 2)}</code>
        </pre>
      </details>
      <Button onClick={() => void loadScans()}>Inspect private scan attachments</Button>
      {scans && (
        <div>
          {scans.length ? (
            scans.map((scan) => (
              <p key={scan.id}>
                <Button onClick={() => void download(scan.id, scan.side)}>
                  Download {scan.side}
                </Button>{" "}
                <span className="badge">{scan.rightsStatus.replaceAll("_", " ")}</span>
              </p>
            ))
          ) : (
            <p>No scans attached.</p>
          )}
        </div>
      )}
      <p className="field-hint">
        Approval publishes factual metadata only. Attached scans remain private.
      </p>
      <Field label="Review reason" hint="Required for the audit trail. At least 5 characters.">
        {(id, hint) => (
          <textarea
            id={id}
            aria-labelledby={`${id}-label`}
            aria-describedby={hint}
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            rows={2}
            minLength={5}
            maxLength={2000}
          />
        )}
      </Field>
      <div className="button-row">
        <Button
          tone="primary"
          disabled={pending || reason.trim().length < 5}
          onClick={() => void review("approve")}
        >
          Approve metadata
        </Button>
        <Button
          tone="danger"
          disabled={pending || reason.trim().length < 5}
          onClick={() => void review("reject")}
        >
          Reject
        </Button>
      </div>
      {message && <output>{message}</output>}
    </article>
  );
}
