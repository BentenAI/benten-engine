//! Mermaid flowchart renderer (diag feature).
//!
//! Output format: `flowchart TD` with one line per node (`nodeId(["LABEL"])`)
//! and one line per edge (`from -->|label| to`). Node ids are sanitized to
//! strip characters Mermaid rejects; the primitive kind is upper-cased into
//! the label so the rendered diagram shows `READ`, `WRITE`, etc.
//!
//! The authoritative Mermaid-parse check runs in the TS Vitest suite (G8);
//! the Rust side asserts the output starts with `flowchart ` and contains
//! at least one `-->` via `exit_criteria_all_six::exit_5_mermaid_output_parses_minimal_shape`.

use crate::{OperationNode, PrimitiveKind, Subgraph};

/// Render a [`Subgraph`] as a Mermaid flowchart string.
///
/// The output starts with `flowchart TD` (top-down) and lists every node
/// followed by every edge. Labels are the primitive kind name (READ,
/// WRITE, …) so a dev pasting the output into a Mermaid preview immediately
/// sees the subgraph shape.
#[must_use]
pub fn render(sg: &Subgraph) -> String {
    let mut out = String::with_capacity(256);
    out.push_str("flowchart TD\n");

    for n in &sg.nodes {
        out.push_str("    ");
        out.push_str(&sanitize_id(&n.id));
        out.push_str("([\"");
        out.push_str(kind_label(n.kind));
        out.push_str(": ");
        out.push_str(&escape_label(&n.id));
        out.push_str("\"])\n");
    }

    for (from, to, label) in &sg.edges {
        out.push_str("    ");
        out.push_str(&sanitize_id(from));
        out.push_str(" -->");
        if !label.is_empty() && label != "next" {
            out.push_str("|");
            out.push_str(&escape_label(label));
            out.push_str("|");
        }
        out.push(' ');
        out.push_str(&sanitize_id(to));
        out.push('\n');
    }

    out
}

/// Primitive-kind label used in the rendered node.
fn kind_label(k: PrimitiveKind) -> &'static str {
    match k {
        PrimitiveKind::Read => "READ",
        PrimitiveKind::Write => "WRITE",
        PrimitiveKind::Transform => "TRANSFORM",
        PrimitiveKind::Branch => "BRANCH",
        PrimitiveKind::Iterate => "ITERATE",
        PrimitiveKind::Wait => "WAIT",
        PrimitiveKind::Call => "CALL",
        PrimitiveKind::Respond => "RESPOND",
        PrimitiveKind::Emit => "EMIT",
        PrimitiveKind::Sandbox => "SANDBOX",
        PrimitiveKind::Subscribe => "SUBSCRIBE",
        PrimitiveKind::Stream => "STREAM",
    }
}

/// Produce a Mermaid-safe identifier (ASCII letters + digits + underscores).
fn sanitize_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'n');
    }
    out
}

/// Escape characters Mermaid labels reject.
fn escape_label(raw: &str) -> String {
    raw.replace('"', "'").replace('\n', " ")
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test-only expectations")]
mod tests {
    use super::*;
    use crate::{OperationNode, Subgraph};

    fn fixture() -> Subgraph {
        Subgraph {
            handler_id: "h".into(),
            nodes: vec![
                OperationNode::new("start", PrimitiveKind::Read),
                OperationNode::new("done", PrimitiveKind::Respond),
            ],
            edges: vec![("start".into(), "done".into(), "ok".into())],
        }
    }

    #[test]
    fn render_starts_with_flowchart() {
        let out = render(&fixture());
        assert!(out.starts_with("flowchart "));
    }

    #[test]
    fn render_contains_edge_arrow() {
        assert!(render(&fixture()).contains("-->"));
    }

    #[test]
    fn render_has_one_line_per_node_and_edge() {
        let out = render(&fixture());
        // Header + 2 nodes + 1 edge + trailing newline.
        assert_eq!(out.lines().count(), 4);
    }

    #[test]
    fn sanitize_strips_non_alnum() {
        assert_eq!(sanitize_id("foo:bar"), "foo_bar");
        assert_eq!(sanitize_id("123"), "n123");
    }

    #[test]
    fn render_contains_kind_labels() {
        let out = render(&fixture());
        assert!(out.contains("READ"));
        assert!(out.contains("RESPOND"));
    }
}
