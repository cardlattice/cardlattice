// @vitest-environment jsdom
import { afterEach, expect, it } from "vitest";
import { cleanup, render } from "@testing-library/react";
import LanguageIcon from "./LanguageIcon.tsx";

afterEach(cleanup);
it.each(["constructor", "__proto__", "unknown"])(
  "uses the globe for unknown language %s",
  (language) => {
    const { container } = render(<LanguageIcon language={language} />);
    expect(container.querySelector("svg")).not.toBeNull();
    expect(container.querySelector("[data-country]")).toBeNull();
  },
);
