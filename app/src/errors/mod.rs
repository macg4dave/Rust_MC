use handlebars::Handlebars;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::OnceLock;
use toml::Value;

/// Parse a TOML string and extract the `[errors]` table into a String map.
///
/// The parser is permissive: values that are not strings are converted to
/// their textual representation. If parsing fails or the table is missing,
/// an empty map is returned. This approach provides robustness against
/// small formatting mistakes while preserving human-editability.
fn parse_templates_from_str(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    match raw.parse::<Value>() {
        Ok(val) => {
            if let Some(errors) = val.get("errors") {
                if let Some(table) = errors.as_table() {
                    for (k, v) in table.iter() {
                        let s = v
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| v.to_string());
                        map.insert(k.clone(), s);
                    }
                }
            }
        }
        Err(e) => {
            // Log parse errors for visibility; the application will fall back to
            // built-in defaults when templates cannot be loaded.
            tracing::warn!(error = ?e, "Failed to parse errors_output.toml");
        }
    }

    map
}

/// Global, lazily-initialised templates map. Using `OnceLock` avoids reparsing the
/// embedded text on every `render_io_error` call and is safe for concurrent use.
static TEMPLATES: OnceLock<HashMap<String, String>> = OnceLock::new();

fn templates() -> &'static HashMap<String, String> {
    TEMPLATES.get_or_init(|| {
        const RAW: &str = include_str!("errors_output.toml");
        parse_templates_from_str(RAW)
    })
}

/// Helper: return either the template from the map or the provided default.
fn template_or_default(key: &str, default: &str) -> String {
    templates()
        .get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

/// Simple placeholder formatter.
///
/// Replaces `{name}` placeholders in `tmpl` with values from `pairs`.
///
/// This is intentionally tiny and explicit: the templates used by the CLI are
/// small and well-known, so a full templating dependency would be unnecessary.
/// The function accepts `tmpl` as `&str` to avoid forcing a clone by callers —
/// callers can pass either a `String` (by reference) or a `&str`.
static HB: OnceLock<Handlebars<'static>> = OnceLock::new();

fn handlebars() -> &'static Handlebars<'static> {
    HB.get_or_init(Handlebars::new)
}

fn format_template(tmpl: &str, pairs: &[(&str, &str)]) -> String {
    // Build a serde_json object to use as template context.
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), JsonValue::String(v.to_string()));
    }
    let ctx = JsonValue::Object(map);

    // Render using the global Handlebars registry. We use `render_template`
    // which renders an ad-hoc template string without registering it first.
    handlebars()
        .render_template(tmpl, &ctx)
        .unwrap_or_else(|err| {
            tracing::warn!(error = ?err, "template render failed — falling back");
            tmpl.to_string()
        })
}

/// Render an error into a user-friendly string. The function will first try to use
/// templates embedded from `errors_output`. If a template isn't present, it falls
/// back to a sensible default.
///
/// Parameters (all optional except `err`):
/// - `path`: primary path involved in the error
/// - `src` / `dst`: source/destination paths for move operations
pub fn render_io_error(
    err: &std::io::Error,
    path: Option<&str>,
    src: Option<&str>,
    dst: Option<&str>,
) -> String {
    use std::io::ErrorKind;

    match err.kind() {
        ErrorKind::NotFound => {
            let tmpl = template_or_default("path_not_found", "Path not found: {path}");
            format_template(&tmpl, &[("path", path.unwrap_or("<unknown>"))])
        }
        ErrorKind::PermissionDenied => {
            let tmpl = template_or_default("permission_denied", "Permission denied: {path}");
            format_template(&tmpl, &[("path", path.unwrap_or("<unknown>"))])
        }
        ErrorKind::AlreadyExists => {
            let tmpl = template_or_default("already_exists", "Target already exists: {path}");
            format_template(
                &tmpl,
                &[("path", path.or(src).or(dst).unwrap_or("<unknown>"))],
            )
        }
        ErrorKind::InvalidInput => {
            let tmpl = template_or_default("invalid_input", "Invalid input: {details}");
            let details = format!("{}", err);
            format_template(&tmpl, &[("details", &details)])
        }
        ErrorKind::BrokenPipe
        | ErrorKind::UnexpectedEof
        | ErrorKind::WouldBlock
        | ErrorKind::TimedOut => {
            let tmpl = template_or_default("io_error", "I/O error: {err}");
            let err_s = format!("{}", err);
            format_template(&tmpl, &[("err", &err_s)])
        }
        _ => {
            // For other errors, attempt to map a move-specific template, then generic.
            if let (Some(s), Some(d)) = (src, dst) {
                let tmpl =
                    template_or_default("unable_to_move", "Unable to move {src} to {dst} ({err})");
                let err_s = format!("{}", err);
                return format_template(&tmpl, &[("src", s), ("dst", d), ("err", &err_s)]);
            }

            // Fallback to a generic I/O error template
            let tmpl = template_or_default("io_error", "I/O error: {err}");
            let err_s = format!("{}", err);
            format_template(&tmpl, &[("err", &err_s)])
        }
    }
}
