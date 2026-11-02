import { useId, type ReactNode } from "react";

export function Field({
  label,
  hint,
  error,
  children,
}: {
  label: string;
  hint?: string;
  error?: string;
  children: (id: string, descriptionId: string | undefined) => ReactNode;
}) {
  const id = useId();
  const descriptionId = hint || error ? `${id}-hint` : undefined;
  return (
    <div className="field">
      <label id={`${id}-label`} htmlFor={id}>
        {label}
      </label>
      {children(id, descriptionId)}
      {(hint || error) && (
        <p id={descriptionId} className={error ? "field-error" : "field-hint"}>
          {error || hint}
        </p>
      )}
    </div>
  );
}
