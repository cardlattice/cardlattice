import { useState } from "react";
import { request } from "../api";
import type { ContributionController } from "./useContributionForm";
import { Field } from "./Field";
import { Button } from "./Button";

export function ContributionSuccess({ controller }: { controller: ContributionController }) {
  const { form, submissionId, setSubmissionId } = controller;
  const [uploadStatus, setUploadStatus] = useState("");
  const [uploading, setUploading] = useState(false);
  async function upload(side: "front" | "back", file: File) {
    setUploading(true);
    setUploadStatus("Uploading a private review scan…");
    try {
      if (file.size > 8 * 1024 * 1024) throw new Error("Choose a PNG or JPEG smaller than 8 MiB.");
      await request(`/v1/contributions/${submissionId}/scans/${side}`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${form.getFieldValue("token")}`,
          "Content-Type": file.type,
        },
        body: file,
      });
      setUploadStatus(
        `${side === "front" ? "Front" : "Back"} scan stored privately. It will not be published with the metadata.`,
      );
    } catch (cause) {
      setUploadStatus(cause instanceof Error ? cause.message : "Upload failed");
    } finally {
      setUploading(false);
    }
  }
  return (
    <section className="panel submission-success">
      <output className="eyebrow">QUEUED FOR HUMAN REVIEW</output>
      <h2>Your contribution is in.</h2>
      <p>
        Submission <code className="break-code">{submissionId}</code>
      </p>
      <p>
        Metadata becomes public only after approval. You can optionally attach front and back scans
        for private review.
      </p>
      {(["front", "back"] as const).map((side) => (
        <Field
          key={side}
          label={`${side === "front" ? "Front" : "Back"} scan (optional)`}
          hint="PNG or JPEG · up to 8 MiB · one per side · originals remain private"
        >
          {(id, hint) => (
            <input
              id={id}
              aria-labelledby={`${id}-label`}
              aria-describedby={hint}
              type="file"
              accept="image/png,image/jpeg"
              disabled={uploading}
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void upload(side, file);
              }}
            />
          )}
        </Field>
      ))}
      <output>{uploadStatus}</output>
      <Button
        onClick={() => {
          setSubmissionId("");
          form.reset();
        }}
      >
        Submit another record
      </Button>
    </section>
  );
}
