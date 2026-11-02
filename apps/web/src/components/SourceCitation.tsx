import type { SourceView } from "../generated/catalogue";

// These two importer-owned descriptions contain audit notes, not attribution text.
// Other contributions retain their full description, including any required credit.
function importedCitation(source: SourceView) {
  if (source.method !== "permitted_official_reference") return null;
  if (
    source.referenceUrl?.startsWith(
      "https://d1i787aglh9bmb.cloudfront.net/assets/img/me-expansions/",
    ) &&
    source.description.startsWith(
      "Names and collector numbers independently transcribed from the publisher's English checklist, main numbered section only.",
    )
  ) {
    return "Pokémon — official card checklist";
  }
  if (
    source.referenceUrl?.startsWith("https://press.pokemon.com/en/products/") &&
    source.description.startsWith("The Pokémon Company International official product information.")
  ) {
    return "Pokémon — official set information";
  }
  return null;
}

export function SourceCitation({ source }: { source: SourceView }) {
  const citation = importedCitation(source);
  return (
    <article className="source">
      {!citation && <p>{source.description}</p>}
      {source.referenceUrl && (
        <a href={source.referenceUrl} target="_blank" rel="noreferrer">
          {citation || "Open source reference"} ↗
        </a>
      )}
      {!citation && (
        <p className="field-hint">
          {source.method.replaceAll("_", " ")} · {source.license}
        </p>
      )}
    </article>
  );
}
