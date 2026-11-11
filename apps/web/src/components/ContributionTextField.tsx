import type { ContributionController } from "./useContributionForm";
import { Field } from "./Field";

export function ContributionTextField({
  form,
  name,
  label,
  type = "text",
}: {
  form: ContributionController["form"];
  name:
    | "token"
    | "name"
    | "slug"
    | "collectorNumber"
    | "language"
    | "region"
    | "rarity"
    | "releaseDate"
    | "referenceUrl";
  label: string;
  type?: string;
}) {
  return (
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
}
