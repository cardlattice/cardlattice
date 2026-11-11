import { useForm } from "@tanstack/react-form";
import { useRef, useState } from "react";
import { z } from "zod";
import { useApi, request, authorized, type PageData, type RecordData } from "../api";

export function useContributionForm({
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
  const idempotency = useRef({
    body: "",
    key: Array.from(crypto.getRandomValues(new Uint8Array(16)), (byte) =>
      byte.toString(16).padStart(2, "0"),
    ).join(""),
  });
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
          idempotency.current = {
            body: serialized,
            key: Array.from(crypto.getRandomValues(new Uint8Array(16)), (byte) =>
              byte.toString(16).padStart(2, "0"),
            ).join(""),
          };
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
  return {
    form,
    games,
    sets,
    cards,
    series,
    gameIdentifier,
    setGameIdentifier,
    error,
    submissionId,
    setSubmissionId,
  };
}

export type ContributionController = ReturnType<typeof useContributionForm>;
