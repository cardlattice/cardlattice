/* oxlint-disable react-doctor/jsx-no-jsx-as-prop -- RecordTile visual is a ReactNode slot; its artwork depends on this record. */
import { useCatalogue } from "./catalogue-provider";
import { languageName } from "./data";
import { useRouterState } from "@tanstack/react-router";
import { RecordLink } from "./record-link";
import type { Printing } from "./data";

import { CardArt } from "./card-art";
export function PrintingTile({ card }: { card: Printing }) {
  const { setName } = useCatalogue();
  const location = useRouterState({ select: (state) => state.location });
  const search = location.search as Record<string, unknown>;
  const context =
    location.pathname.includes("/printings/") || location.pathname.includes("/cards/")
      ? search
      : { origin: location.pathname, browse: search };
  const visual = <CardArt card={card} />;
  return (
    <RecordLink
      to={"/pokemon/printings/" + card.id}
      search={context}

      title={card.name}
      eyebrow={setName(card.setId) + " · " + card.number}
      description={(card.rarity ?? "Rarity not recorded") + " · " + languageName(card.language)}
      visual={visual}
    />
  );
}
