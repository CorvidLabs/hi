//! `hi view` writes a page a product person can read.
//!
//! The raw files are for whoever writes the intent. This is for everyone else:
//! the prose up front, the criteria as a plain readable list, and the ids small
//! and out of the way. Self-contained HTML, no network, no build step.

use std::fs;

use anyhow::{Context, Result};

use crate::doc::{Criterion, Doc};
use crate::workspace::Workspace;

/// Escape the five characters that matter in HTML text.
fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Render the inline markdown a criterion sentence is allowed to carry:
/// `code`, **bold**, *italic*, and [links](url). Everything is escaped first,
/// so no sentence can inject markup.
pub fn inline_markdown(raw: &str) -> String {
    let escaped = escape(raw);
    let chars: Vec<char> = escaped.chars().collect();
    let mut out = String::with_capacity(escaped.len());
    let mut index = 0;

    while index < chars.len() {
        // `code` wins over everything else inside it.
        if chars[index] == '`'
            && let Some(close) = find_from(&chars, index + 1, '`')
        {
            let inner: String = chars[index + 1..close].iter().collect();
            out.push_str(&format!("<code>{inner}</code>"));
            index = close + 1;
            continue;
        }

        // [text](url)
        if chars[index] == '['
            && let Some(text_end) = find_from(&chars, index + 1, ']')
            && chars.get(text_end + 1) == Some(&'(')
            && let Some(url_end) = find_from(&chars, text_end + 2, ')')
        {
            let text: String = chars[index + 1..text_end].iter().collect();
            let url: String = chars[text_end + 2..url_end].iter().collect();
            // Only http(s) and relative links; anything else renders as text.
            if url.starts_with("http://") || url.starts_with("https://") || url.starts_with('/') {
                out.push_str(&format!("<a href=\"{url}\">{text}</a>"));
                index = url_end + 1;
                continue;
            }
        }

        // **bold** before *italic*, so the longer marker wins.
        if chars[index] == '*' && chars.get(index + 1) == Some(&'*') {
            if let Some(close) = find_pair(&chars, index + 2) {
                let inner: String = chars[index + 2..close].iter().collect();
                out.push_str(&format!("<strong>{inner}</strong>"));
                index = close + 2;
                continue;
            }
        } else if chars[index] == '*'
            && let Some(close) = find_from(&chars, index + 1, '*')
        {
            let inner: String = chars[index + 1..close].iter().collect();
            out.push_str(&format!("<em>{inner}</em>"));
            index = close + 1;
            continue;
        }

        out.push(chars[index]);
        index += 1;
    }

    out
}

fn find_from(chars: &[char], start: usize, target: char) -> Option<usize> {
    (start..chars.len()).find(|&i| chars[i] == target)
}

/// Find a closing `**` at or after `start`.
fn find_pair(chars: &[char], start: usize) -> Option<usize> {
    (start..chars.len().saturating_sub(1)).find(|&i| chars[i] == '*' && chars[i + 1] == '*')
}

/// Remove HTML comment spans. The template hi writes into every new file has
/// one, and a comment that starts mid-paragraph would otherwise render as
/// literal text on the page a product person reads.
fn strip_comments(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(open) = rest.find("<!--") {
        out.push_str(&rest[..open]);
        match rest[open..].find("-->") {
            Some(close) => rest = &rest[open + close + 3..],
            // An unterminated comment swallows the remainder, as HTML would.
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Render prose as paragraphs, keeping blank-line separation.
fn paragraphs(raw: &str) -> String {
    strip_comments(raw)
        .split("\n\n")
        .map(|block| block.trim().to_string())
        .filter(|block| !block.is_empty())
        .map(|block| format!("<p>{}</p>", inline_markdown(&block.replace('\n', " "))))
        .collect::<Vec<_>>()
        .join("\n")
}

fn criteria_list(criteria: &[Criterion]) -> String {
    let mut out = String::new();
    for criterion in criteria {
        let depth = criterion.id.as_ref().map(|id| id.depth()).unwrap_or(1);
        // Show who is speaking. Without it every criterion reads as the same
        // undifferentiated "I", and a reader cannot tell whether the person is
        // being paid or doing the paying.
        let role = criterion
            .role
            .as_ref()
            .map(|r| format!("<span class=\"role\">{}</span>", escape(r)))
            .unwrap_or_default();
        let sentence = if criterion.role.is_some() {
            crate::doc::without_role(&criterion.text)
        } else {
            criterion.text.clone()
        };
        out.push_str(&format!(
            // The newline between the spans is deliberate: without whitespace
            // there, copying from the page yields "SEND-1I hit enter". Flex
            // layout ignores it, so nothing moves.
            "<li class=\"d{}\">\n<span class=\"cid\">{}</span>\n<span class=\"ctext\">{role}{}</span>\n</li>\n",
            depth.min(4),
            escape(&criterion.raw_id),
            inline_markdown(&sentence),
        ));
    }
    out
}

fn feature_section(doc: &Doc) -> String {
    let title = doc.title.clone().unwrap_or_else(|| doc.name());
    let mut out = String::new();

    out.push_str("<section class=\"feature\">\n");
    out.push_str(&format!("<h2>{}</h2>\n", escape(&title)));

    let intent = paragraphs(&doc.intent);
    if !intent.is_empty() {
        out.push_str(&format!("<div class=\"intent\">{intent}</div>\n"));
    }

    if doc.criteria.is_empty() {
        out.push_str("<p class=\"empty\">Nothing written down yet.</p>\n");
    } else {
        out.push_str(&format!(
            "<ul class=\"criteria\">\n{}</ul>\n",
            criteria_list(&doc.criteria)
        ));
    }

    if !doc.retired.is_empty() {
        out.push_str("<details class=\"retired\"><summary>");
        out.push_str(&format!("{} retired", doc.retired.len()));
        out.push_str("</summary>\n");
        out.push_str(&format!(
            "<ul class=\"criteria\">\n{}</ul>\n",
            criteria_list(&doc.retired)
        ));
        out.push_str("</details>\n");
    }

    out.push_str("</section>\n");
    out
}

/// Build the whole page.
pub fn render(workspace: &Workspace, product_intent: Option<&str>) -> String {
    let title = workspace
        .root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Intent".to_string());

    let total: usize = workspace.criteria_count();
    let features = workspace
        .docs
        .iter()
        .map(feature_section)
        .collect::<Vec<_>>()
        .join("\n");

    let lead = product_intent
        .map(paragraphs)
        .filter(|rendered| !rendered.is_empty())
        .map(|rendered| format!("<div class=\"lead\">{rendered}</div>"))
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}: intent</title>
<style>
:root {{
  color-scheme: light;
  --ink: #15181B; --paper: #FAF9F6; --surface: #FFFFFF;
  --sheen: #0E6F66; --sheen-strong: #0B5750;
  --text-muted: #34383D; --text-faint: #50555B;
  --hairline: rgba(21,24,27,0.12); --wash: rgba(21,24,27,0.06);
  /* System stack on purpose: the page must not fetch anything, so opening it
     never discloses the reader's address to a font host (hi: VIEW-2). */
  --font-display: "Schibsted Grotesk", ui-sans-serif, system-ui, -apple-system,
    "Segoe UI", Helvetica, Arial, sans-serif;
  --font-mono: "Spline Sans Mono", ui-monospace, SFMono-Regular, Menlo,
    Consolas, monospace;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    color-scheme: dark;
    --ink: #F4F3EF; --paper: #131619; --surface: #1B1F23;
    --sheen: #45D0BC; --sheen-strong: #45D0BC;
    --text-muted: #C8C6BE; --text-faint: #A3A199;
    --hairline: rgba(244,243,239,0.15); --wash: rgba(244,243,239,0.08);
  }}
}}
:root[data-theme="dark"] {{
  color-scheme: dark;
  --ink: #F4F3EF; --paper: #131619; --surface: #1B1F23;
  --sheen: #45D0BC; --sheen-strong: #45D0BC;
  --text-muted: #C8C6BE; --text-faint: #A3A199;
  --hairline: rgba(244,243,239,0.15); --wash: rgba(244,243,239,0.08);
}}
* {{ box-sizing: border-box; }}
body {{
  background: var(--paper); color: var(--ink);
  font-family: var(--font-display); font-size: 17px; line-height: 1.6;
  margin: 0; padding-block: 0 80px; padding-left: 20px; padding-right: 20px;
  -webkit-font-smoothing: antialiased;
}}
.wrap {{ max-width: 760px; margin: 0 auto; }}
header {{ padding-block: 56px 0; }}
.eyebrow {{
  font-family: var(--font-mono); font-size: 12px; letter-spacing: .1em;
  text-transform: uppercase; color: var(--sheen-strong); margin: 0 0 18px;
}}
h1 {{
  font-size: clamp(34px, 6vw, 52px); font-weight: 900; letter-spacing: -.03em;
  line-height: 1.04; margin: 0 0 20px; text-wrap: balance;
}}
.lead p {{ font-size: 19px; line-height: 1.55; color: var(--text-muted); margin: 0 0 16px; }}
.rule {{
  height: 5px; border: 0; border-radius: 3px; margin: 40px 0 0;
  background: linear-gradient(90deg, #0E6F66 0%, #1799A3 55%, #1E6FA8 100%);
}}
.feature {{ padding-block: 44px 0; }}
h2 {{
  font-size: clamp(24px, 3.6vw, 30px); font-weight: 700; letter-spacing: -.02em;
  margin: 0 0 12px; text-wrap: balance;
}}
.intent p {{ color: var(--text-muted); margin: 0 0 14px; max-width: 64ch; }}
ul.criteria {{ list-style: none; margin: 20px 0 0; padding: 0; }}
ul.criteria li {{
  display: flex; gap: 12px; align-items: baseline;
  padding: 9px 0; border-top: 1px solid var(--hairline);
}}
ul.criteria li.d2 {{ padding-left: 26px; }}
ul.criteria li.d3 {{ padding-left: 52px; }}
ul.criteria li.d4 {{ padding-left: 78px; }}
.cid {{
  font-family: var(--font-mono); font-size: 11.5px; color: var(--text-faint);
  white-space: nowrap; flex: 0 0 auto; padding-top: 3px; min-width: 76px;
}}
.ctext {{ flex: 1 1 auto; }}
code {{
  font-family: var(--font-mono); font-size: .87em;
  background: var(--wash); padding: 1px 5px; border-radius: 3px;
}}
a {{ color: var(--sheen); text-underline-offset: 2px; }}
.empty {{ color: var(--text-faint); font-style: italic; }}
details.retired {{ margin-top: 20px; }}
details.retired summary {{
  cursor: pointer; font-family: var(--font-mono); font-size: 12px;
  letter-spacing: .06em; text-transform: uppercase; color: var(--text-faint);
}}
details.retired .cid, details.retired .ctext {{ opacity: .6; }}
footer {{
  margin-top: 56px; padding-top: 20px; border-top: 1px solid var(--hairline);
  font-size: 13.5px; color: var(--text-faint);
}}
@media (max-width: 560px) {{
  ul.criteria li {{ flex-direction: column; gap: 3px; }}
  ul.criteria li.d2 {{ padding-left: 16px; }}
  ul.criteria li.d3 {{ padding-left: 32px; }}
  ul.criteria li.d4 {{ padding-left: 48px; }}
}}
</style>
</head>
<body>
<div class="wrap">
<header>
<p class="eyebrow">What this should be</p>
<h1>{title}</h1>
{lead}
<hr class="rule">
</header>
{features}
<footer>
<p>{total} criteria across {count} {noun}. Generated from the <code>hi/</code> files by <code>hi view</code>.</p>
<p>Every line here is something a person asked for, written as what this should be rather than as a report of what it currently does. A line can describe something true today, something a year out, or something that has since been revised. These are directions, and they stay correct through a wrong turn.</p>
<p>The short code beside each one is its permanent id. You can quote it and it will still mean this line later.</p>
</footer>
</div>
</body>
</html>
"#,
        title = escape(&title),
        lead = lead,
        features = features,
        total = total,
        count = workspace.docs.len(),
        noun = if workspace.docs.len() == 1 {
            "feature"
        } else {
            "features"
        },
    )
}

/// Write the page, defaulting to `intent.html` at the repository root.
pub fn write(workspace: &Workspace, out: Option<&str>) -> Result<String> {
    let product_intent = fs::read_to_string(workspace.intent_path())
        .ok()
        .map(strip_index);
    let html = render(workspace, product_intent.as_deref());

    let path = match out {
        Some(out) => workspace.root.join(out),
        None => workspace.root.join("intent.html"),
    };
    fs::write(&path, html).with_context(|| format!("writing {}", path.display()))?;
    Ok(workspace.rel(&path))
}

/// Drop the generated index block and the H1 from INTENT.md prose.
fn strip_index(raw: String) -> String {
    let mut out = Vec::new();
    let mut skipping = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "<!-- hi:index -->" {
            skipping = true;
            continue;
        }
        if trimmed == "<!-- /hi:index -->" {
            skipping = false;
            continue;
        }
        if skipping || trimmed.starts_with("# ") || trimmed == "## Features" {
            continue;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Doc;
    use std::path::PathBuf;

    fn workspace(files: &[(&str, &str)]) -> Workspace {
        let docs = files
            .iter()
            .map(|(name, raw)| Doc::parse(PathBuf::from(format!("/r/hi/{name}")), raw))
            .collect();
        Workspace {
            root: PathBuf::from("/r"),
            dir: PathBuf::from("/r/hi"),
            docs,
        }
    }

    #[test]
    fn renders_inline_code_bold_italic_and_links() {
        assert_eq!(inline_markdown("a `code` b"), "a <code>code</code> b");
        assert_eq!(inline_markdown("a **bold** b"), "a <strong>bold</strong> b");
        assert_eq!(inline_markdown("a *italic* b"), "a <em>italic</em> b");
        assert_eq!(
            inline_markdown("see [docs](https://x.com/y)"),
            "see <a href=\"https://x.com/y\">docs</a>"
        );
    }

    #[test]
    fn escapes_markup_in_a_sentence() {
        assert_eq!(
            inline_markdown("a <script>alert(1)</script> b"),
            "a &lt;script&gt;alert(1)&lt;/script&gt; b"
        );
        // A quote in prose must not break an attribute.
        assert!(inline_markdown("he said \"no\"").contains("&quot;no&quot;"));
    }

    #[test]
    fn refuses_a_javascript_link_and_leaves_it_as_text() {
        let rendered = inline_markdown("[click](javascript:alert(1))");
        assert!(!rendered.contains("<a href"), "{rendered}");
    }

    #[test]
    fn leaves_an_unclosed_marker_alone() {
        assert_eq!(inline_markdown("2 * 3 is 6"), "2 * 3 is 6");
        assert_eq!(inline_markdown("a `b"), "a `b");
    }

    #[test]
    fn html_comments_never_reach_the_page() {
        assert_eq!(strip_comments("a <!-- hidden --> b"), "a  b");
        assert_eq!(strip_comments("<!-- only a comment -->").trim(), "");
        // An unterminated comment swallows the rest, as a browser would.
        assert_eq!(strip_comments("keep <!-- rest is gone"), "keep ");
    }

    #[test]
    fn a_feature_whose_prose_is_only_the_template_comment_renders_no_empty_block() {
        let raw = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Billing\n\n## Intent\n\n<!-- What is this for? -->\n\n## Criteria\n\nSEND-1  One.\n";
        let html = render(&workspace(&[("billing.md", raw)]), None);
        assert!(
            !html.contains("What is this for"),
            "the prompt must not ship"
        );
        assert!(
            !html.contains("<div class=\"intent\"></div>"),
            "no empty block"
        );
        assert!(html.contains("SEND-1"));
    }

    #[test]
    fn the_page_fetches_nothing() {
        // "no network" is a headline promise, and the page is meant to be
        // sendable to anyone. It must not disclose the reader to a third party.
        let chat = "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n";
        let html = render(&workspace(&[("chat.md", chat)]), None);
        for probe in [
            "http://", "https://", "//fonts.", "<script", "<iframe", "@import",
        ] {
            assert!(!html.contains(probe), "the page must not reference {probe}");
        }
    }

    #[test]
    fn copied_text_keeps_the_id_and_sentence_apart() {
        // Without whitespace between the spans, copying from the page, or any
        // html-to-text conversion, yields "SEND-1I hit enter".
        let chat =
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  I hit enter.\n";
        let html = render(&workspace(&[("chat.md", chat)]), None);
        assert!(
            html.contains("<span class=\"cid\">SEND-1</span>\n<span class=\"ctext\">"),
            "there must be whitespace between the id and the sentence"
        );
        // And nowhere on the page may two spans touch, which is the shape
        // that makes copied text run together.
        assert!(
            !html.contains("</span><span"),
            "no two spans may sit adjacent with no whitespace"
        );
    }

    #[test]
    fn page_carries_intent_prose_and_every_criterion() {
        let chat = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nIt should feel like texting.\n\n## Criteria\n\nSEND-1  I hit enter and it shows up.\nSEND-1.a  If I have no connection it queues.\n";
        let html = render(&workspace(&[("chat.md", chat)]), Some("The product why."));
        assert!(html.contains("It should feel like texting."));
        assert!(html.contains("The product why."));
        assert!(html.contains("SEND-1.a"));
        assert!(html.contains("<h2>Chat</h2>"));
        // Cases are visually nested.
        assert!(html.contains("class=\"d2\""));
        // Both themes are defined, per the brand system.
        assert!(html.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn retired_criteria_are_tucked_away_but_present() {
        let chat = "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  Live.\n\n## Retired\n\nSEND-2  Gone.\n        retired: different product\n";
        let html = render(&workspace(&[("chat.md", chat)]), None);
        assert!(html.contains("<details"));
        assert!(html.contains("1 retired"));
        assert!(html.contains("SEND-2"));
    }

    #[test]
    fn strips_the_generated_index_from_product_prose() {
        let raw = "# Product\n\nThe why.\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md)\n<!-- /hi:index -->\n";
        assert_eq!(strip_index(raw.to_string()), "The why.");
    }
}
