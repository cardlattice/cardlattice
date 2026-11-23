// @vitest-environment jsdom
import { expect, it, vi } from "vitest";
const mocks = vi.hoisted(() => ({ render: vi.fn(), create: vi.fn() }));
vi.mock("react-dom/client", () => ({
  createRoot: mocks.create.mockReturnValue({ render: mocks.render }),
}));
it("mounts the dashboard with a query provider", async () => {
  document.body.innerHTML = '<div id="root"></div>';
  await import("./main.tsx");
  expect(mocks.create).toHaveBeenCalledWith(document.getElementById("root"));
  expect(mocks.render).toHaveBeenCalledOnce();
});

it("reports a missing mounting element", async () => {
  vi.resetModules();
  document.body.replaceChildren();
  await expect(import("./main.tsx")).rejects.toThrow("Missing dashboard root element");
});
