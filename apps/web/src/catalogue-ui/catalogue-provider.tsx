import { createContext, use, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { useRouterState } from "@tanstack/react-router";
import * as stylex from "@stylexjs/stylex";
import { Theme, useSystemTheme } from "@imlunahey/components/theme";
import { Panel } from "@imlunahey/components/panel";
import { Button } from "@imlunahey/components/button";
import { createCatalogue, type Catalogue } from "./data";
import { loadSnapshot } from "./snapshot";
const context = createContext<Catalogue | null>(null);
const styles = stylex.create({
  loading: { maxWidth: 600, marginInline: "auto", paddingBlock: 80, paddingInline: 20 },
});
export function CatalogueProvider({ children }: { children: ReactNode }) {
  const mode = useSystemTheme();
  const path = useRouterState({ select: (state) => state.location.pathname });
  const needsCatalogue =
    path === "/" ||
    path === "/search" ||
    path === "/data" ||
    path === "/coverage" ||
    path === "/pokemon" ||
    path.startsWith("/pokemon/");
  const { data, isError, error, refetch } = useQuery({
    queryKey: ["cardlattice-catalogue"],
    queryFn: async ({ signal }) => createCatalogue(await loadSnapshot(signal)),
    staleTime: 5 * 60 * 1000,
    retry: 1,
    refetchOnWindowFocus: false,
    enabled: needsCatalogue,
  });
  if (!data && needsCatalogue)
    return (
      <Theme mode={mode}>
        <main {...stylex.props(styles.loading)}>
          <Panel title={isError ? "Catalogue unavailable" : "Loading Cardlattice"}>
            <p role={isError ? "alert" : "status"}>
              {isError ? error?.message : "Downloading and verifying the published catalogue…"}
            </p>
            {isError && <Button onClick={() => void refetch()}>Try again</Button>}
          </Panel>
        </main>
      </Theme>
    );
  return <context.Provider value={data ?? null}>{children}</context.Provider>;
}
export function useCatalogueCount() {
  return use(context)?.printings.length;
}
export function useCatalogue() {
  const value = use(context);
  if (!value) throw Error("CatalogueProvider is required.");
  return value;
}
