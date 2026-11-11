import type { ContributionController } from "./useContributionForm";
import type { RecordData } from "../api";
import { ContributionTextField } from "./ContributionTextField";
import { Field } from "./Field";

export function ContributionRecordFields({
  controller,
  original,
}: {
  controller: ContributionController;
  original?: RecordData;
}) {
  const { form, games, sets, cards, series, gameIdentifier, setGameIdentifier } = controller;
  return (
    <>
      <h2>
        01 <span>Your access & record</span>
      </h2>
      <ContributionTextField form={form} name="token" label="Contributor token" type="password" />
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
      <ContributionTextField form={form} name="name" label="Record name" />
      <form.Subscribe selector={(state) => state.values.kind}>
        {(kind) => (
          <>
            {kind === "game" && (
              <ContributionTextField form={form} name="slug" label="Stable game slug" />
            )}
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
                  <ContributionTextField
                    form={form}
                    name="collectorNumber"
                    label="Collector number"
                  />
                  <ContributionTextField form={form} name="language" label="Language tag" />
                </div>
                <div className="form-grid">
                  <ContributionTextField form={form} name="rarity" label="Rarity (optional)" />
                  <ContributionTextField form={form} name="region" label="Region (optional)" />
                </div>
              </>
            )}
          </>
        )}
      </form.Subscribe>
      <ContributionTextField
        form={form}
        name="releaseDate"
        label="Release date (optional)"
        type="date"
      />
      <form.Field name="attributes">
        {(field) => (
          <Field
            label="Game-specific attributes (JSON)"
            hint={
              'For Pokémon cards: {"category":"pokemon","hp":40,"types":["Grass"],"stage":"Basic"}. Leave {} when not recorded.'
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
    </>
  );
}
