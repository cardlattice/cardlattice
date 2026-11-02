import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { QueryState } from "./QueryState";

describe("QueryState", () => {
  it("announces pending data without fabricated content", () => {
    render(
      <QueryState pending error={null}>
        <p>Records</p>
      </QueryState>,
    );
    expect(screen.getByRole("status")).toHaveAccessibleName("Loading catalogue");
    expect(screen.queryByText("Records")).not.toBeInTheDocument();
  });
  it("lets users retry a failed request", () => {
    const retry = vi.fn();
    render(
      <QueryState pending={false} error={new Error("Database unavailable")} retry={retry}>
        Records
      </QueryState>,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Database unavailable");
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(retry).toHaveBeenCalledOnce();
  });
});
