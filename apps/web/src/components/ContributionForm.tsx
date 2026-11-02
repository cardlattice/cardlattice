import { useForm } from "@tanstack/react-form";
import { Link } from "@tanstack/react-router";
import { useRef, useState } from "react";
import { z } from "zod";
import { useApi, request, authorized, type PageData, type RecordData } from "../api";
import { Field } from "./Field";
import { Button } from "./Button";

export function ContributionForm({
  original,
  initialGame = "pokemon",
}: {
  original?: RecordData;
  initialGame?: string;
}) {
  const games = useApi<PageData>("/v1/games");
  const [gameIdentifier, setGameIdentifier] = useState(initialGame);
  const sets = useApi<PageData>(`/v1/games/${gameIdentifier}/sets?limit=100`);
  const cards = useApi<PageData>(`/v1/games/${gameIdentifier}/cards?limit=100`);
  const series = useApi<PageData>(`/v1/games/${gameIdentifier}/series?limit=100`);
  const [error, setError] = useState("");
  const [submissionId, setSubmissionId] = useState("");
  const [uploadStatus, setUploadStatus] = useState("");
  const [uploading, setUploading] = useState(false);
  const idempotency = useRef({ body: "", key: crypto.randomUUID() });
  const schema = z.object({
    name: z.string().trim().min(1, "Provide a name").max(200),
    token: z.string().min(32, "Enter your contributor token"),
    source: z
      .string()
      .trim()
      .min(10, "Describe your original source in at least 10 characters")
      .max(2000),
    terms: z.literal(true, { error: "Accept the contribution terms" }),
  });
  const form = useForm({
    defaultValues: {
      token: "",
      kind: original?.kind || "card",
      name: original?.name || "",
      operation: original ? "update" : "create",
      slug: "",
      seriesId: original?.seriesId || "",
      setId: original?.setId || "",
      cardId: original?.cardId || "",
      collectorNumber: original?.collectorNumber || "",
      language: original?.language || "en",
      region: original?.region || "",
      rarity: original?.rarity || "",
      releaseDate: original?.releaseDate || "",
      attributes: JSON.stringify(original?.attributes || {}, null, 2),
      source: "",
      referenceUrl: "",
      method: "original_contribution",
      synthetic: original?.synthetic ?? false,
      terms: false,
      website: "",
    },
    onSubmit: async ({ value }) => {
      setError("");
      const valid = schema.safeParse(value);
      if (!valid.success) {
        setError(valid.error.issues.map((issue) => issue.message).join(". "));
        return;
      }
      try {
        const gameId =
          original?.gameId ||
          games.data?.data.find(
            (game) => game.slug === gameIdentifier || game.id === gameIdentifier,
          )?.id;
        const draft = {
          kind: value.kind,
          name: value.name,
          gameId: value.kind === "game" ? null : gameId,
          slug: value.kind === "game" ? value.slug : null,
          seriesId: value.kind === "set" ? value.seriesId || null : null,
          setId: value.kind === "printing" ? value.setId : null,
          cardId: value.kind === "printing" ? value.cardId : null,
          collectorNumber: value.kind === "printing" ? value.collectorNumber : null,
          language: value.kind === "printing" ? value.language : null,
          region: value.region || null,
          rarity: value.rarity || null,
          releaseDate: value.releaseDate || null,
          attributes: JSON.parse(value.attributes),
          synthetic: value.synthetic,
          source: {
            description: value.source,
            referenceUrl: value.referenceUrl || null,
            method: value.method,
            license: "CC0-1.0",
          },
        };
        const body = {
          operation: value.operation,
          targetId: original?.id || null,
          baseUpdatedAt: original?.updatedAt || null,
          draft,
          termsAccepted: value.terms,
          website: value.website,
        };
        const serialized = JSON.stringify(body);
        if (idempotency.current.body !== serialized)
          idempotency.current = { body: serialized, key: crypto.randomUUID() };
        const options = authorized(value.token, body);
        const response = await request<{ id: string }>("/v1/contributions", {
          ...options,
          headers: { ...options.headers, "Idempotency-Key": idempotency.current.key },
        });
        setSubmissionId(response.id);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : "Submission failed");
      }
    },
  });
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
  const textField = (
    name:
      | "token"
      | "name"
      | "slug"
      | "collectorNumber"
      | "language"
      | "region"
      | "rarity"
      | "releaseDate"
      | "referenceUrl",
    label: string,
    type = "text",
  ) => (
    <form.Field name={name}>
      {(field) => (
        <Field label={label}>
          {(id) => (
            <input
              id={id}
              aria-labelledby={`${id}-label`}
              type={type}
              value={field.state.value}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
              required={["token", "name"].includes(name)}
              autoComplete={name === "token" ? "off" : undefined}
              maxLength={name === "referenceUrl" ? 2000 : 200}
            />
          )}
        </Field>
      )}
    </form.Field>
  );
  if (submissionId)
    return (
      <section className="panel submission-success" role="status">
        <p className="eyebrow">QUEUED FOR HUMAN REVIEW</p>
        <h2>Your contribution is in.</h2>
        <p>
          Submission <code className="break-code">{submissionId}</code>
        </p>
        <p>
          Metadata becomes public only after approval. You can optionally attach front and back
          scans for private review.
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
        <p role="status">{uploadStatus}</p>
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
  return (
    <form
      className="contribution-form"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit();
      }}
    >
      <h2>
        01 <span>Your access & record</span>
      </h2>
      {textField("token", "Contributor token", "password")}
      <div className="form-grid">
        <form.Field name="kind">
          {(field) => (
            <Field label="Record type">
              {(id) => (
                <select
                  id={id}
                  aria-labelledby={`${id}-label`}
                  value={field.state.value}
                  disabled={!!original}
                  onChange={(event) => field.handleChange(event.target.value)}
                >
                  {["game", "series", "set", "card", "printing"].map((kind) => (
                    <option key={kind} value={kind}>
                      {kind === "card" ? "Card identity" : kind[0].toUpperCase() + kind.slice(1)}
                    </option>
                  ))}
                </select>
              )}
            </Field>
          )}
        </form.Field>
        <Field label="Game">
          {(id) => (
            <select
              id={id}
              aria-labelledby={`${id}-label`}
              value={
                games.data?.data.find(
                  (game) => game.id === gameIdentifier || game.slug === gameIdentifier,
                )?.id || gameIdentifier
              }
              disabled={!!original}
              onChange={(event) => setGameIdentifier(event.target.value)}
            >
              {games.data?.data.map((game) => (
                <option value={game.id} key={game.id}>
                  {game.name}
                </option>
              ))}
            </select>
          )}
        </Field>
      </div>
      {original && (
        <form.Field name="operation">
          {(field) => (
            <Field label="Correction type">
              {(id) => (
                <select
                  id={id}
                  aria-labelledby={`${id}-label`}
                  value={field.state.value}
                  onChange={(event) => field.handleChange(event.target.value)}
                >
                  <option value="update">Update metadata</option>
                  <option value="delete">Request removal</option>
                </select>
              )}
            </Field>
          )}
        </form.Field>
      )}
      {textField("name", "Record name")}
      <form.Subscribe selector={(state) => state.values.kind}>
        {(kind) => (
          <>
            {kind === "game" && textField("slug", "Stable game slug")}
            {kind === "set" && (
              <form.Field name="seriesId">
                {(field) => (
                  <Field label="Series (optional)">
                    {(id) => (
                      <select
                        id={id}
                        aria-labelledby={`${id}-label`}
                        value={field.state.value}
                        onChange={(event) => field.handleChange(event.target.value)}
                      >
                        <option value="">No series</option>
                        {series.data?.data.map((record) => (
                          <option key={record.id} value={record.id}>
                            {record.name}
                          </option>
                        ))}
                      </select>
                    )}
                  </Field>
                )}
              </form.Field>
            )}
            {kind === "printing" && (
              <>
                <div className="form-grid">
                  {(["setId", "cardId"] as const).map((name) => (
                    <form.Field key={name} name={name}>
                      {(field) => (
                        <Field label={name === "setId" ? "Set" : "Card identity"}>
                          {(id) => (
                            <select
                              id={id}
                              aria-labelledby={`${id}-label`}
                              required
                              value={field.state.value}
                              onChange={(event) => field.handleChange(event.target.value)}
                            >
                              <option value="">Choose a record</option>
                              {(name === "setId" ? sets : cards).data?.data.map((record) => (
                                <option key={record.id} value={record.id}>
                                  {record.name}
                                </option>
                              ))}
                            </select>
                          )}
                        </Field>
                      )}
                    </form.Field>
                  ))}
                </div>
                <div className="form-grid">
                  {textField("collectorNumber", "Collector number")}
                  {textField("language", "Language tag")}
                </div>
                <div className="form-grid">
                  {textField("rarity", "Rarity (optional)")}
                  {textField("region", "Region (optional)")}
                </div>
              </>
            )}
          </>
        )}
      </form.Subscribe>
      {textField("releaseDate", "Release date (optional)", "date")}
      <form.Field name="attributes">
        {(field) => (
          <Field
            label="Game-specific attributes (JSON)"
            hint={
              'For Pokémon cards: {"category":"pokemon","hp":40,"types":["grass"],"stage":"basic"}. Leave {} when not recorded.'
            }
          >
            {(id, hint) => (
              <textarea
                id={id}
                aria-labelledby={`${id}-label`}
                aria-describedby={hint}
                rows={5}
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
                maxLength={8192}
                spellCheck={false}
                className="code-input"
              />
            )}
          </Field>
        )}
      </form.Field>
      <form.Field name="synthetic">
        {(field) => (
          <label className="checkbox">
            <input
              type="checkbox"
              checked={field.state.value}
              onChange={(event) => field.handleChange(event.target.checked)}
            />{" "}
            This is a synthetic development record, not a real product.
          </label>
        )}
      </form.Field>
      <h2>
        02 <span>Source & rights</span>
      </h2>
      <form.Field name="method">
        {(field) => (
          <Field label="How was this metadata collected?">
            {(id) => (
              <select
                id={id}
                aria-labelledby={`${id}-label`}
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
              >
                <option value="original_contribution">Original contribution</option>
                <option value="physical_inspection">Inspection of a physical card I own</option>
                <option value="permitted_official_reference">Permitted official reference</option>
              </select>
            )}
          </Field>
        )}
      </form.Field>
      <form.Field name="source">
        {(field) => (
          <Field
            label="Source description"
            hint="Describe your own observations or the permitted reference. Do not paste another catalogue or API response."
          >
            {(id, hint) => (
              <textarea
                id={id}
                aria-labelledby={`${id}-label`}
                aria-describedby={hint}
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
                minLength={10}
                maxLength={2000}
                required
                rows={4}
              />
            )}
          </Field>
        )}
      </form.Field>
      {textField("referenceUrl", "Official reference URL (optional)", "url")}
      <div className="honeypot" aria-hidden="true">
        <form.Field name="website">
          {(field) => (
            <label>
              Leave empty
              <input
                tabIndex={-1}
                autoComplete="off"
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
              />
            </label>
          )}
        </form.Field>
      </div>
      <form.Field name="terms">
        {(field) => (
          <label className="checkbox">
            <input
              type="checkbox"
              required
              checked={field.state.value}
              onChange={(event) => field.handleChange(event.target.checked)}
            />
            <span>
              I independently sourced this metadata, have the right to contribute it, and dedicate
              my original metadata contribution under CC0-1.0. Scans are for private review only. I
              accept the <Link to="/licensing">contributor terms</Link>.
            </span>
          </label>
        )}
      </form.Field>
      {error && (
        <p className="notice error" role="alert">
          {error}
        </p>
      )}
      <form.Subscribe selector={(state) => state.isSubmitting}>
        {(pending) => (
          <Button type="submit" tone="primary" disabled={pending}>
            {pending ? "Submitting…" : "Submit for review →"}
          </Button>
        )}
      </form.Subscribe>
    </form>
  );
}
