import SelectPicker from "./SelectPicker.tsx";
import Icon from "./Icon.tsx";
import { useEffect, useSyncExternalStore } from "react";
import { useHydrated } from "../hooks/useHydrated.ts";

let memoryTheme = "auto";
const key = "cardlattice-theme";
function readTheme() {
  let saved = memoryTheme;
  try {
    saved = localStorage.getItem(key) ?? "auto";
  } catch {
    /* Storage can be disabled. */
  }
  return saved === "light" || saved === "dark" ? saved : "auto";
}
function subscribe(listener: () => void) {
  window.addEventListener("storage", listener);
  window.addEventListener("cardlattice-theme-change", listener);
  return () => {
    window.removeEventListener("storage", listener);
    window.removeEventListener("cardlattice-theme-change", listener);
  };
}
const serverTheme = () => "auto";
export default function ThemePicker() {
  const ready = useHydrated();
  const theme = useSyncExternalStore(subscribe, readTheme, serverTheme);
  useEffect(() => {
    if (theme === "auto") delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = theme;
  }, [theme]);
  function changeTheme(value: string) {
    memoryTheme = value;
    try {
      localStorage.setItem(key, value);
    } catch {
      /* Theme still works in memory. */
    }
    window.dispatchEvent(new Event("cardlattice-theme-change"));
  }
  return (
    <div className="theme-label" hidden={!ready}>
      <span>Theme</span>
      <SelectPicker
        label="Theme"
        value={theme}
        onChange={changeTheme}
        options={[
          { value: "auto", label: "Auto", icon: <Icon name="monitor" /> },
          { value: "light", label: "Light", icon: <Icon name="sun" /> },
          { value: "dark", label: "Dark", icon: <Icon name="moon" /> },
        ]}
      />
    </div>
  );
}
