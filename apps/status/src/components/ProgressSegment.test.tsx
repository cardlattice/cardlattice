// @vitest-environment jsdom
import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Tooltip } from "@base-ui/react/tooltip";
import ProgressSegment from "./ProgressSegment.tsx";

afterEach(cleanup);
it("shows counts and percentage on hover and dismisses when the pointer leaves", async () => {
  const user = userEvent.setup();
  render(
    <Tooltip.Provider delay={0}>
      <svg>
        <ProgressSegment status="partial" label="In Progress" count={2} total={3} x={0} />
      </svg>
    </Tooltip.Provider>,
  );
  const segment = screen.getByRole("img", { name: "In Progress: 2 of 3 sets (66.7%)" });
  await user.hover(segment);
  expect((await screen.findByRole("tooltip")).textContent).toBe("In Progress: 2 of 3 sets (66.7%)");
  await user.unhover(segment);
  await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
});
it("makes the tooltip available by keyboard and dismissible with Escape", async () => {
  const user = userEvent.setup();
  render(
    <svg>
      <ProgressSegment status="done" label="Done" count={1} total={1} x={0} />
    </svg>,
  );
  await user.tab();
  expect(document.activeElement).toBe(screen.getByRole("img", { name: "Done: 1 of 1 set (100%)" }));
  expect((await screen.findByRole("tooltip")).textContent).toBe("Done: 1 of 1 set (100%)");
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
});
