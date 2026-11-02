import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, request, authorized } from "./api";

afterEach(() => vi.unstubAllGlobals());
describe("API boundary", () => {
  it("preserves the API error without inventing fallback data", async () => {
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          new Response(
            JSON.stringify({ error: { code: "not_found", message: "Resource not found" } }),
            { status: 404 },
          ),
        ),
    );
    await expect(request("/v1/games/missing")).rejects.toEqual(
      new ApiError(404, "Resource not found"),
    );
  });
  it("public reads never add authentication", async () => {
    const fetch = vi.fn().mockResolvedValue(new Response('{"data":[]}'));
    vi.stubGlobal("fetch", fetch);
    await request("/v1/games");
    expect(fetch).toHaveBeenCalledWith("/v1/games", {});
    expect(authorized("test-token").headers).toEqual({ Authorization: "Bearer test-token" });
  });
});
