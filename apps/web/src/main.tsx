import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
  Link,
} from "@tanstack/react-router";
import { Layout } from "./components/Layout";
import "./styles.css";

const root = createRootRoute({
  component: Layout,
  notFoundComponent: () => (
    <div className="page empty">
      <h1>Page not found</h1>
      <p>This address doesn’t point to a catalogue page.</p>
      <Link to="/games" className="button button-primary">
        Browse the catalogue
      </Link>
    </div>
  ),
  errorComponent: ({ error, reset }) => (
    <div className="page notice error" role="alert">
      <h1>Something went wrong</h1>
      <p>{error instanceof Error ? error.message : "An unexpected error occurred."}</p>
      <button type="button" className="button" onClick={reset}>
        Try again
      </button>
    </div>
  ),
});
const route = (path: string, component: ReturnType<typeof lazyRouteComponent>) =>
  createRoute({
    getParentRoute: () => root,
    path,
    component,
    validateSearch: (search: Record<string, unknown>) => ({
      game: typeof search.game === "string" ? search.game : "pokemon",
      q: typeof search.q === "string" ? search.q : "",
      cursor: typeof search.cursor === "string" ? search.cursor : "",
      language: typeof search.language === "string" ? search.language : "",
      setId: typeof search.setId === "string" ? search.setId : "",
      cardId: typeof search.cardId === "string" ? search.cardId : "",
      productId: typeof search.productId === "string" ? search.productId : "",
      collectionId: typeof search.collectionId === "string" ? search.collectionId : "",
      artworkId: typeof search.artworkId === "string" ? search.artworkId : "",
      sort: search.sort === "newest" ? "newest" : "name",
      kind: typeof search.kind === "string" ? search.kind : "",
      target: typeof search.target === "string" ? search.target : "",
    }),
  });
const routes = [
  route(
    "/",
    lazyRouteComponent(() => import("./pages/Home"), "Home"),
  ),
  ...[
    "/games",
    "/games/$game",
    "/games/$game/sets",
    "/games/$game/cards",
    "/games/$game/printings",
    "/games/$game/products",
    "/games/$game/collections",
    "/games/$game/artworks",

    "/search",
  ].map((path) =>
    route(
      path,
      lazyRouteComponent(() => import("./pages/Browse"), "Browse"),
    ),
  ),
  ...[
    "/games/$game/sets/$id",
    "/games/$game/cards/$id",
    "/games/$game/printings/$id",
    "/games/$game/products/$id",
    "/games/$game/collections/$id",
    "/games/$game/artworks/$id",

    "/games/$game/series/$id",
  ].map((path) =>
    route(
      path,
      lazyRouteComponent(() => import("./pages/Detail"), "Detail"),
    ),
  ),
  route(
    "/docs",
    lazyRouteComponent(() => import("./pages/Docs"), "Docs"),
  ),
  route(
    "/datasets",
    lazyRouteComponent(() => import("./pages/Datasets"), "Datasets"),
  ),
  route(
    "/coverage",
    lazyRouteComponent(() => import("./pages/Coverage"), "Coverage"),
  ),
  route(
    "/status",
    lazyRouteComponent(() => import("./pages/Status"), "Status"),
  ),
  route(
    "/changelog",
    lazyRouteComponent(() => import("./pages/Changelog"), "Changelog"),
  ),
  route(
    "/contribute",
    lazyRouteComponent(() => import("./pages/Contribute"), "Contribute"),
  ),
  route(
    "/moderate",
    lazyRouteComponent(() => import("./pages/Moderate"), "Moderate"),
  ),
  ...["/licensing", "/about"].map((path) =>
    route(
      path,
      lazyRouteComponent(() => import("./pages/Information"), "Information"),
    ),
  ),
];
const router = createRouter({ routeTree: root.addChildren(routes), defaultPreload: "intent" });
const client = new QueryClient();
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
);
