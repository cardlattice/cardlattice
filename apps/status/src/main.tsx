import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRoot } from "react-dom/client";
import Dashboard from "./components/Dashboard.tsx";
import "./style.css";

const client = new QueryClient();
const root = document.getElementById("root");
if (!root) {
  throw new Error("Missing dashboard root element");
}
createRoot(root).render(
  <QueryClientProvider client={client}>
    <Dashboard />
  </QueryClientProvider>,
);
