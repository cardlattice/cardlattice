import { Link } from "@tanstack/react-router";
import type { ContributionController } from "./useContributionForm";
import { ContributionTextField } from "./ContributionTextField";
import { Field } from "./Field";

export function ContributionSourceFields({ form }: { form: ContributionController["form"] }) {
  return (
    <>
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
      <ContributionTextField
        form={form}
        name="referenceUrl"
        label="Official reference URL (optional)"
        type="url"
      />
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
    </>
  );
}
