import { Link, Outlet, useRouterState } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { Layers3, ArrowUpRight, Menu } from "lucide-react";
import { ThemeToggle } from "./ThemeToggle";

const navigation = [
  { to: "/games", label: "Catalogue" },
  { to: "/docs", label: "Developers" },
  { to: "/datasets", label: "Datasets" },
  { to: "/contribute", label: "Contribute" },
];
export function Layout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const main = useRef<HTMLElement>(null);
  const previousPath = useRef(pathname);
  useEffect(() => {
    const changed = previousPath.current !== pathname;
    previousPath.current = pathname;
    document.title = `${document.querySelector("h1")?.textContent || "Open card data"} — Cardlattice`;
    if (changed) {
      main.current?.focus();
      window.scrollTo({ top: 0 });
    }
  }, [pathname]);
  return (
    <>
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <section className="announcement" aria-label="About Cardlattice">
        Independent. Open source. Free to build on.{" "}
        <Link to="/about">
          Meet the project <ArrowUpRight size={13} aria-hidden="true" />
        </Link>
      </section>
      <header className="site-header">
        <div className="header-inner">
          <Link to="/" className="wordmark" aria-label="Cardlattice home">
            <span className="brand-mark">
              <Layers3 size={23} aria-hidden="true" />
            </span>
            cardlattice<span className="brand-dot">.</span>
          </Link>
          <nav aria-label="Main navigation" className="desktop-nav">
            {navigation.map((item) => (
              <Link
                key={item.to}
                to={item.to}
                activeProps={{ className: "active", "aria-current": "page" }}
              >
                {item.label}
              </Link>
            ))}
          </nav>
          <div className="header-actions">
            <Link to="/search" className="search-link">
              Search
            </Link>
            <ThemeToggle />
            <details className="mobile-nav">
              <summary aria-label="Open navigation">
                <Menu size={22} aria-hidden="true" />
              </summary>
              <nav aria-label="Mobile navigation">
                {navigation.map((item) => (
                  <Link key={item.to} to={item.to}>
                    {item.label}
                  </Link>
                ))}
              </nav>
            </details>
          </div>
        </div>
      </header>
      <main id="main" tabIndex={-1} ref={main}>
        <Outlet />
      </main>
      <footer className="site-footer">
        <div>
          <Link to="/" className="wordmark">
            cardlattice<span className="brand-dot">.</span>
          </Link>
          <p>
            A shared foundation for card data.
            <br />
            Independently maintained, always open.
          </p>
        </div>
        <nav aria-label="Footer navigation">
          <a href="https://status.cardlattice.com">Coverage</a>
          <a href="https://status.cardlattice.com">Status</a>
          <Link to="/changelog">Changelog</Link>
          <Link to="/licensing">Licensing</Link>
          <Link to="/about">About & governance</Link>
          <Link to="/docs">API — coming soon</Link>
        </nav>
        <p className="footer-note">
          Software: MIT · Original metadata: CC0
          <br />
          Not affiliated with any game publisher.
        </p>
      </footer>
    </>
  );
}
