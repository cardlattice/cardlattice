use serde_json::{Value, json};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Cardlattice public API",
        version = "1.0.0",
        description = "Free, independently curated trading card metadata. Public GET and HEAD requests never require authentication."
    ),
    components(schemas(
        catalogue::Record,
        catalogue::Page,
        catalogue::Draft,
        catalogue::Kind,
        catalogue::SourceInput,
        catalogue::SourceMethod,
        catalogue::PokemonAttributes,
        crate::reads::RecordDetail,
        crate::reads::SourceView,
        crate::reads::ImageView,
        crate::reads::VariantView,
        crate::reads::LocalisationView,
        crate::reads::Stats,
        publishing::Manifest,
        crate::error::ErrorBody,
        crate::error::ErrorDetail,
        crate::contributions::Submission,
        crate::contributions::Operation,
        crate::contributions::Review,
        crate::contributions::Contribution
    ))
)]
struct ApiDoc;

pub fn document() -> Value {
    let mut document = serde_json::to_value(ApiDoc::openapi()).expect("serializable OpenAPI");
    document["openapi"] = json!("3.1.0");
    document["servers"] = json!([{"url":"/","description":"Current origin"}]);
    document["components"]["securitySchemes"] = json!({"contributorToken":{"type":"http","scheme":"bearer","description":"Maintainer-issued contributor capability; never needed for public reads"},"moderatorToken":{"type":"http","scheme":"bearer"}});
    let paths = document["paths"].as_object_mut().expect("paths object");
    for (path, schema, summary, list) in [
        ("/v1/games", "Page", "List supported games", true),
        ("/v1/games/{game}", "Record", "Get a game", false),
        ("/v1/games/{game}/series", "Page", "List series", true),
        (
            "/v1/games/{game}/series/{id}",
            "RecordDetail",
            "Get a series",
            false,
        ),
        ("/v1/games/{game}/sets", "Page", "List sets", true),
        (
            "/v1/games/{game}/sets/{id}",
            "RecordDetail",
            "Get a set",
            false,
        ),
        (
            "/v1/games/{game}/cards",
            "Page",
            "List card identities",
            true,
        ),
        (
            "/v1/games/{game}/cards/{id}",
            "RecordDetail",
            "Get a card identity",
            false,
        ),
        (
            "/v1/games/{game}/printings",
            "Page",
            "List distinct printings",
            true,
        ),
        (
            "/v1/games/{game}/printings/{id}",
            "RecordDetail",
            "Get a printing",
            false,
        ),
        (
            "/v1/search",
            "Page",
            "Search independently curated metadata",
            true,
        ),
        ("/v1/stats", "Stats", "Measured catalogue coverage", false),
        (
            "/v1/datasets",
            "Datasets",
            "List the latest 50 immutable snapshots",
            false,
        ),
        (
            "/v1/status",
            "Status",
            "Persisted origin probe measurements",
            false,
        ),
        (
            "/v1/changelog",
            "Changelog",
            "Latest 50 catalogue revisions",
            false,
        ),
        ("/healthz", "Health", "Process liveness", false),
        ("/readyz", "Health", "Database readiness", false),
    ] {
        let mut parameters = Vec::new();
        if path.contains("{game}") {
            parameters.push(json!({"name":"game","in":"path","required":true,"schema":{"type":"string"},"example":"pokemon"}));
        }
        if path.contains("{id}") {
            parameters
                .push(json!({"name":"id","in":"path","required":true,"schema":{"type":"string"}}));
        }
        if list {
            for (name, schema) in [
                (
                    "limit",
                    json!({"type":"integer","minimum":1,"maximum":100,"default":20}),
                ),
                ("cursor", json!({"type":"string","maxLength":2048})),
                ("q", json!({"type":"string","maxLength":120})),
                (
                    "sort",
                    json!({"type":"string","enum":["name","newest"],"default":"name"}),
                ),
                ("language", json!({"type":"string","example":"en"})),
                ("setId", json!({"type":"string"})),
                ("cardId", json!({"type":"string"})),
                ("rarity", json!({"type":"string"})),
                ("game", json!({"type":"string"})),
            ] {
                if name == "game" && path != "/v1/search" {
                    continue;
                }
                parameters.push(json!({"name":name,"in":"query","required":false,"schema":schema}));
            }
        } else if path.contains("{id}") {
            parameters.push(json!({"name":"language","in":"query","schema":{"type":"string"}}));
        }
        parameters.push(json!({"name":"If-None-Match","in":"header","schema":{"type":"string"}}));
        let response_schema = if schema == "Record" {
            json!({"type":"object","required":["data"],"properties":{"data":{"$ref":"#/components/schemas/CatalogueRecord"}}})
        } else {
            json!({"$ref":format!("#/components/schemas/{schema}")})
        };
        let operation = json!({"summary":summary,"tags":["Public catalogue"],"security":[],"parameters":parameters,"responses":{
            "200":{"description":"Success","headers":{"ETag":{"schema":{"type":"string"}},"Cache-Control":{"schema":{"type":"string"}},"Last-Modified":{"schema":{"type":"string"}}},"content":{"application/json":{"schema":response_schema}}},
            "304":{"description":"Representation unchanged"},"308":{"description":"Canonical query URL; follow Location"},
            "400":{"$ref":"#/components/responses/Error"},"404":{"$ref":"#/components/responses/Error"},"429":{"$ref":"#/components/responses/Error"},"503":{"$ref":"#/components/responses/Error"}
        }});
        let mut head = operation.clone();
        head["summary"] = json!(format!("{summary} (headers only)"));
        head["responses"]["200"]
            .as_object_mut()
            .unwrap()
            .remove("content");
        paths.insert(path.into(), json!({"get":operation,"head":head}));
    }
    for (path, method, summary, request, role) in [
        (
            "/v1/contributions",
            "post",
            "Submit an original draft",
            "Submission",
            "contributorToken",
        ),
        (
            "/v1/contributions/{id}",
            "get",
            "Read your submission and review history",
            "",
            "contributorToken",
        ),
        (
            "/v1/moderation/contributions",
            "get",
            "Pending review queue",
            "",
            "moderatorToken",
        ),
        (
            "/v1/moderation/contributions/{id}/review",
            "post",
            "Approve or reject a draft atomically",
            "Review",
            "moderatorToken",
        ),
        (
            "/v1/moderation/outbox",
            "get",
            "Inspect latest invalidations",
            "",
            "moderatorToken",
        ),
        (
            "/v1/moderation/outbox/{id}/replay",
            "post",
            "Replay a failed invalidation",
            "",
            "moderatorToken",
        ),
    ] {
        let mut parameters = Vec::new();
        if path.contains("{id}") {
            parameters
                .push(json!({"name":"id","in":"path","required":true,"schema":{"type":"string"}}));
        }
        if path == "/v1/contributions" {
            parameters.push(json!({"name":"Idempotency-Key","in":"header","required":true,"schema":{"type":"string"}}));
        }
        if path == "/v1/moderation/contributions" {
            parameters.push(json!({"name":"cursor","in":"query","schema":{"type":"string"}}));
        }
        let mut operation = json!({"summary":summary,"tags":["Contributions"],"security":[{role:[]}],"parameters":parameters,"responses":{"200":{"description":"Success","content":{"application/json":{"schema":{"type":"object"}}}},"400":{"$ref":"#/components/responses/Error"},"401":{"$ref":"#/components/responses/Error"},"409":{"$ref":"#/components/responses/Error"},"429":{"$ref":"#/components/responses/Error"}}});
        if !request.is_empty() {
            operation["requestBody"] = json!({"required":true,"content":{"application/json":{"schema":{"$ref":format!("#/components/schemas/{request}")}}}});
        }
        paths.insert(path.into(), json!({method:operation}));
    }
    paths.insert("/v1/contributions/{id}/scans/{side}".into(),json!({"post":{"summary":"Upload a private review scan","security":[{"contributorToken":[]}],"parameters":[{"name":"id","in":"path","required":true,"schema":{"type":"string"}},{"name":"side","in":"path","required":true,"schema":{"type":"string","enum":["front","back"]}}],"requestBody":{"required":true,"content":{"image/png":{"schema":{"type":"string","format":"binary"}},"image/jpeg":{"schema":{"type":"string","format":"binary"}}}},"responses":{"200":{"description":"Stored privately"},"400":{"$ref":"#/components/responses/Error"},"401":{"$ref":"#/components/responses/Error"},"413":{"$ref":"#/components/responses/Error"}}}}));
    paths.insert("/v1/moderation/scans/{id}".into(),json!({"get":{"summary":"Download private scan for moderation","security":[{"moderatorToken":[]}],"parameters":[{"name":"id","in":"path","required":true,"schema":{"type":"string"}}],"responses":{"200":{"description":"Private attachment; never cache","content":{"image/png":{"schema":{"type":"string","format":"binary"}},"image/jpeg":{"schema":{"type":"string","format":"binary"}}}},"401":{"$ref":"#/components/responses/Error"},"404":{"$ref":"#/components/responses/Error"}}}}));
    paths.insert("/datasets/{filename}".into(),json!({"get":{"summary":"Download immutable metadata snapshot","security":[],"parameters":[{"name":"filename","in":"path","required":true,"schema":{"type":"string"}}],"responses":{"200":{"description":"Newline-delimited JSON records; no artwork","content":{"application/x-ndjson":{"schema":{"type":"string","format":"binary"}}}},"304":{"description":"Unchanged"},"404":{"$ref":"#/components/responses/Error"}}}}));
    document["components"]["responses"] = json!({"Error":{"description":"Consistent error response","content":{"application/json":{"schema":{"$ref":"#/components/schemas/ErrorBody"}}}}});
    let schemas = document["components"]["schemas"].as_object_mut().unwrap();
    schemas.insert("Datasets".into(),json!({"type":"object","required":["data","license","includesArtwork","limit"],"properties":{"data":{"type":"array","items":{"$ref":"#/components/schemas/Manifest"}},"license":{"type":"string"},"includesArtwork":{"type":"boolean"},"limit":{"type":"integer"}}}));
    schemas.insert(
        "Health".into(),
        json!({"type":"object","required":["status"],"properties":{"status":{"type":"string"}}}),
    );
    schemas.insert("Status".into(),json!({"type":"object","required":["status","measurements"],"properties":{"status":{"type":"string","enum":["unknown","operational","degraded"]},"measurements":{"type":"array","items":{"type":"object"}},"pendingInvalidations":{"type":"integer"},"failedInvalidations":{"type":"integer"}}}));
    schemas.insert("Changelog".into(),json!({"type":"object","required":["data","limit"],"properties":{"data":{"type":"array","items":{"type":"object"}},"limit":{"type":"integer"}}}));
    document
}
