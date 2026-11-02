import { useState } from "react";
import { useApi } from "../api";
import { PageHeader } from "../components/PageHeader";
import { QueryState } from "../components/QueryState";
import { Field } from "../components/Field";
import { Button } from "../components/Button";

type Parameter = {
  name: string;
  in: string;
  required?: boolean;
  schema: { type: string; default?: unknown; example?: unknown };
  example?: unknown;
};
type Operation = {
  summary: string;
  parameters: Parameter[];
  responses: Record<string, unknown>;
  security?: unknown[];
};
type Specification = {
  openapi: string;
  paths: Record<string, { get?: Operation }>;
  components: { schemas: Record<string, unknown> };
};
const examples = {
  curl: 'curl -i "http://localhost:8080/v1/games/pokemon/cards?limit=10"',
  JavaScript:
    'const response = await fetch(\n  "http://localhost:8080/v1/games/pokemon/cards?limit=10"\n);\nif (!response.ok) throw new Error(`HTTP ${response.status}`);\nconst page = await response.json();\nconsole.log(page.data, page.nextCursor);',
  Python:
    'import json\nfrom urllib.request import urlopen\n\nwith urlopen("http://localhost:8080/v1/games/pokemon/cards?limit=10") as response:\n    page = json.load(response)\n    print(page["data"], page["nextCursor"])',
  Rust: '// Dependencies: reqwest (json, rustls-tls), serde_json, tokio (full)\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let page: serde_json::Value = reqwest::get(\n        "http://localhost:8080/v1/games/pokemon/cards?limit=10"\n    ).await?.error_for_status()?.json().await?;\n    println!("{}", page["data"]);\n    Ok(())\n}',
};
export function Docs() {
  const spec = useApi<Specification>("/openapi.json");
  const [path, setPath] = useState("/v1/games");
  const [values, setValues] = useState<Record<string, string>>({ game: "pokemon" });
  const [language, setLanguage] = useState<keyof typeof examples>("curl");
  const [output, setOutput] = useState("Run a request to see its live response.");
  const [pending, setPending] = useState(false);
  const [headers, setHeaders] = useState("");
  const operation = spec.data?.paths[path]?.get;
  async function run() {
    setPending(true);
    try {
      let target = path;
      const query = new URLSearchParams();
      for (const parameter of operation?.parameters || []) {
        const value = values[parameter.name] || "";
        if (parameter.required && !value) throw new Error(`${parameter.name} is required`);
        if (parameter.in === "path")
          target = target.replace(`{${parameter.name}}`, encodeURIComponent(value));
        else if (parameter.in === "query" && value) query.set(parameter.name, value);
      }
      // This explorer deliberately inspects both success and error responses.
      // oxlint-disable-next-line react-doctor/no-fetch-response-used-without-status-check
      const response = await fetch(`${target}${query.size ? `?${query}` : ""}`, {
        headers: values["If-None-Match"] ? { "If-None-Match": values["If-None-Match"] } : {},
      });
      // The explorer intentionally displays error bodies alongside their HTTP status.
      // oxlint-disable-next-line react-doctor/no-fetch-response-used-without-status-check
      const text = await response.text();
      setOutput(text ? JSON.stringify(JSON.parse(text), null, 2) : "(empty response body)");
      setHeaders(
        `HTTP ${response.status}\n${["etag", "last-modified", "cache-control", "cf-cache-status"].map((name) => `${name}: ${response.headers.get(name) || "not present"}`).join("\n")}`,
      );
    } catch (cause) {
      setOutput(cause instanceof Error ? cause.message : "Request failed");
    } finally {
      setPending(false);
    }
  }
  return (
    <div className="page">
      <PageHeader
        eyebrow="SMALL API. OPEN POSSIBILITIES."
        title="Build something with it."
        action={
          <a href="/openapi.json" className="button button-secondary" download="openapi.json">
            Download OpenAPI 3.1 ↗
          </a>
        }
      >
        <p>
          Anonymous reads, stable internal IDs, and predictable JSON. Start with a single request.
        </p>
      </PageHeader>
      <div className="docs-grid">
        <aside className="docs-sidebar">
          <h2>The essentials</h2>
          <a href="#explorer">API explorer</a>
          <a href="#examples">Code examples</a>
          <a href="#pagination">Pagination & filtering</a>
          <a href="#caching">Caching & errors</a>
          <a href="#schemas">Response schemas</a>
          <div className="notice">
            <strong>No API key required.</strong>
            <p>All public GET and HEAD requests work anonymously.</p>
          </div>
        </aside>
        <div>
          <section id="explorer">
            <h2>Interactive API explorer</h2>
            <p className="muted">Generated from the running server’s OpenAPI contract.</p>
            <QueryState pending={spec.isPending} error={spec.error} retry={spec.refetch}>
              <div className="panel">
                <Field label="Public endpoint">
                  {(id) => (
                    <select
                      id={id}
                      aria-labelledby={`${id}-label`}
                      value={path}
                      onChange={(event) => {
                        setPath(event.target.value);
                        setHeaders("");
                        setOutput("Run a request to see its live response.");
                      }}
                    >
                      {Object.entries(spec.data?.paths || {})
                        .filter(
                          ([, item]) =>
                            item.get?.security?.length === 0 &&
                            !item.get?.summary.includes("Download"),
                        )
                        .map(([endpoint, item]) => (
                          <option key={endpoint} value={endpoint}>
                            GET {endpoint} — {item.get?.summary}
                          </option>
                        ))}
                    </select>
                  )}
                </Field>
                <div className="form-grid">
                  {operation?.parameters.map((parameter) => (
                    <Field
                      key={`${parameter.in}-${parameter.name}`}
                      label={`${parameter.name}${parameter.required ? " *" : ""}`}
                      hint={`${parameter.in} · ${parameter.schema.type}${parameter.schema.default !== undefined ? ` · default ${parameter.schema.default}` : ""}`}
                    >
                      {(id, hint) => (
                        <input
                          id={id}
                          aria-labelledby={`${id}-label`}
                          aria-describedby={hint}
                          value={values[parameter.name] || ""}
                          required={parameter.required}
                          onChange={(event) =>
                            setValues({ ...values, [parameter.name]: event.target.value })
                          }
                          placeholder={String(parameter.example || parameter.schema.example || "")}
                        />
                      )}
                    </Field>
                  ))}
                </div>
                <Button tone="primary" disabled={pending} onClick={() => void run()}>
                  {pending ? "Requesting…" : "Send GET request →"}
                </Button>
              </div>
              <div className="response-panel" aria-live="polite">
                {headers && <pre className="response-headers">{headers}</pre>}
                <pre className="code-block">
                  <code>{output}</code>
                </pre>
              </div>
            </QueryState>
          </section>
          <section id="examples" className="section">
            <h2>Your language. One endpoint.</h2>
            <div className="tabs" aria-label="Example language">
              {Object.keys(examples).map((name) => (
                <Button
                  key={name}
                  aria-pressed={language === name}
                  onClick={() => setLanguage(name as keyof typeof examples)}
                >
                  {name}
                </Button>
              ))}
            </div>
            <pre className="code-block">
              <code>{examples[language]}</code>
            </pre>
          </section>
          <section id="pagination" className="section prose">
            <h2>Keep requests predictable.</h2>
            <p>
              Collections return <code>data</code>, <code>nextCursor</code>, and <code>limit</code>.
              Default pages contain up to 20 records; the maximum is 100. Continue with the returned
              cursor and the same filters. A cursor belongs to its original query.
            </p>
            <p>
              Use <code>q</code> for Unicode-normalised search, <code>language</code> for localised
              names and printing language, and <code>sort=name</code> or <code>sort=newest</code>.
              Printing collections also accept <code>setId</code>, <code>cardId</code>, and{" "}
              <code>rarity</code>.
            </p>
            <p>
              Search accepts up to 120 characters and eight terms. Unknown or repeated parameters
              produce a consistent error. Equivalent query URLs redirect to a canonical form.
            </p>
          </section>
          <section id="caching" className="section prose">
            <h2>Cache thoughtfully.</h2>
            <p>
              Keep the response <code>ETag</code> and send it as <code>If-None-Match</code> on your
              next request. An unchanged representation returns <code>304</code>. Browser caching is
              short; shared caching is longer for individual records.
            </p>
            <p>
              Filtered lists and search have bounded 60-second shared TTLs. Moderation,
              contributions, health, and status responses use <code>no-store</code>. Cloudflare is
              optional; <code>CF-Cache-Status</code> is absent when running locally.
            </p>
            <pre className="code-block">
              {'{ "error": { "code": "not_found", "message": "Resource not found" } }'}
            </pre>
            <p>
              Expect 400 for invalid inputs, 404 for missing records, 409 for contribution
              conflicts, 429 for rate limits, and 503 if the database is unavailable. Public reads
              never require registration.
            </p>
          </section>
          <section id="schemas" className="section">
            <h2>Response schemas</h2>
            {Object.entries(spec.data?.components.schemas || {}).map(([name, schema]) => (
              <details key={name} className="schema">
                <summary>{name}</summary>
                <pre className="code-block">
                  <code>{JSON.stringify(schema, null, 2)}</code>
                </pre>
              </details>
            ))}
          </section>
        </div>
      </div>
    </div>
  );
}
