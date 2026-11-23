import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRoot } from "react-dom/client";
import Dashboard from "./components/Dashboard.tsx";
import "./style.css";

const client = new QueryClient();
createRoot(document.getElementById("root")!).render(
  <QueryClientProvider client={client}>
    <Dashboard />
  </QueryClientProvider>,
);
