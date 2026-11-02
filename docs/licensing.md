# Data sourcing, licensing and contributor terms

Software is licensed under [MIT](../LICENSE). Original synthetic development
fixtures and original factual metadata contributions are dedicated under
[CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/). The licence on software
or metadata does not grant rights in card artwork, trademarks or copied text.

## Allowed sourcing

- Original manual entry and observations of user-owned physical cards.
- Original community contributions with a source description.
- Official references where their use is permitted, with recorded permission or
  terms and legally required attribution preserved.

Do not ingest TCGdex, Scryfall, YGOPRODeck, other third-party catalogue dumps, or
their API responses as seed data. Do not evade this rule by renaming columns,
removing provider IDs, or stripping attribution. No external pricing API or
third-party copyrighted images are enabled by default.

Every accepted draft records source description, method, optional reference URL,
licence, and field-level provenance. Source descriptions are public; moderation
credentials, private scan originals, and internal audit identities are not.

## Contributor terms

By submitting, the contributor confirms that:

1. They independently collected the metadata or have permission to contribute it.
2. They dedicate their original factual metadata contribution under CC0-1.0 to the
   extent legally possible. They do not submit copied protected descriptions.
3. They accurately describe the source and relevant rights restrictions.
4. They are authorised to provide any scans privately for review and permit storage
   and inspection for that purpose. This does not grant public artwork rights.
5. Their submission can be reviewed, corrected, rejected, and recorded in audit
   history. They will not include unnecessary personal information.

The first release accepts only metadata compatible with its CC0 export. Sources
that require incompatible redistribution conditions need a separate licensing
implementation before acceptance. Attribution required by law or permission must
never be concealed. The record source view is not a substitute for an image credit.

## Images

Ownership or scanning does not confer artwork redistribution rights. Scans are
private by default and independent of metadata approval. Public image records
require a reviewed rights relation containing permission evidence, licence,
attribution where needed, reviewer, and approval date. There is no automatic scan
publication endpoint. Implementing a dedicated rights-review UI is roadmap work.

No real card artwork is distributed in this repository. The project is not
affiliated with any card publisher. Operators must provide a monitored rights
contact and a documented retention/removal process before public launch. Retain
only scans needed for active review; manually remove rejected/obsolete private
originals under an operator-approved retention policy while retaining non-sensitive
decision evidence. Automated scan retention is not yet implemented.
