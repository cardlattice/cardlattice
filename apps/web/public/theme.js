try {
  document.documentElement.dataset.theme = localStorage.getItem("cardlattice-theme") || "system";
} catch {
  /* System theme remains available without storage. */
}
