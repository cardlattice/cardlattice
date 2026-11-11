import { useState } from "react";
import { Moon, Sun, Monitor } from "lucide-react";
import { Button } from "./Button";

export function ThemeToggle() {
  const [theme, setTheme] = useState(() =>
    typeof document === "undefined" ? "system" : document.documentElement.dataset.theme || "system",
  );
  function cycle() {
    const next = theme === "system" ? "dark" : theme === "dark" ? "light" : "system";
    document.documentElement.dataset.theme = next;
    try {
      localStorage.setItem("cardlattice-theme", next);
    } catch {
      /* Theme still works for this visit. */
    }
    setTheme(next);
  }
  const Icon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;
  return (
    <Button tone="quiet" onClick={cycle} aria-label={`Theme: ${theme}. Change theme`}>
      <Icon size={18} aria-hidden="true" />
      <span className="sr-only">Change theme</span>
    </Button>
  );
}
