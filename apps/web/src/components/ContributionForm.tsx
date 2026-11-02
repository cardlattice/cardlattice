import type { RecordData } from "../api";
import { useContributionForm } from "./useContributionForm";
import { ContributionRecordFields } from "./ContributionRecordFields";
import { ContributionSourceFields } from "./ContributionSourceFields";
import { ContributionSuccess } from "./ContributionSuccess";
import { Button } from "./Button";

export function ContributionForm({
  original,
  initialGame = "pokemon",
}: {
  original?: RecordData;
  initialGame?: string;
}) {
  const controller = useContributionForm({ original, initialGame });
  const { form, submissionId, error } = controller;
  if (submissionId) return <ContributionSuccess controller={controller} />;
  return (
    <form
      className="contribution-form"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit();
      }}
    >
      <ContributionRecordFields controller={controller} original={original} />
      <ContributionSourceFields form={form} />
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
