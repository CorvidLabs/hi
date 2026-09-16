//! `hi view` builds a page a product person can read, search and share.
//!
//! One self-contained file: no network, no CDN, no fonts, no build step. The
//! script is inline, so the page still opens from an email attachment or a USB
//! stick (hi: VIEW-2). With scripting off every criterion is still visible, and
//! the controls stay hidden, because the script only ever hides rows that are
//! already in the document.

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
        if chars[index] == '`'
            && let Some(close) = find_from(&chars, index + 1, '`')
        {
            let inner: String = chars[index + 1..close].iter().collect();
            out.push_str(&format!("<code>{inner}</code>"));
            index = close + 1;
            continue;
        }

        if chars[index] == '['
            && let Some(text_end) = find_from(&chars, index + 1, ']')
            && chars.get(text_end + 1) == Some(&'(')
            && let Some(url_end) = find_from(&chars, text_end + 2, ')')
        {
            let text: String = chars[index + 1..text_end].iter().collect();
            let url: String = chars[text_end + 2..url_end].iter().collect();
            if url.starts_with("http://") || url.starts_with("https://") || url.starts_with('/') {
                out.push_str(&format!("<a href=\"{url}\">{text}</a>"));
                index = url_end + 1;
                continue;
            }
        }

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

fn find_pair(chars: &[char], start: usize) -> Option<usize> {
    (start..chars.len().saturating_sub(1)).find(|&i| chars[i] == '*' && chars[i + 1] == '*')
}

/// Remove `<!-- ... -->` spans so an editing note never reaches the page.
fn strip_comments(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(open) = rest.find("<!--") {
        out.push_str(&rest[..open]);
        match rest[open..].find("-->") {
            Some(close) => rest = &rest[open + close + 3..],
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

/// One row, carrying everything the controls need to search, sort and filter.
fn row(doc: &Doc, criterion: &Criterion, retired: bool) -> String {
    let id = escape(&criterion.raw_id);
    let depth = criterion.id.as_ref().map(|i| i.depth()).unwrap_or(1);
    let family = criterion
        .id
        .as_ref()
        .map(|i| i.family.clone())
        .unwrap_or_default();

    // A lowercased haystack, so searching never has to walk the DOM.
    let haystack =
        escape(&format!("{} {} {}", criterion.raw_id, criterion.text, family).to_lowercase());

    let why = criterion
        .note
        .as_ref()
        .map(|n| format!("\n<span class=\"why\">{}</span>", inline_markdown(n)))
        .unwrap_or_default();

    format!(
        "<li class=\"row d{}\" id=\"{}\" data-id=\"{}\" data-family=\"{}\" \
         data-file=\"{}\" data-retired=\"{}\" data-find=\"{}\">\n\
         <a class=\"cid\" href=\"#{}\" title=\"Link to {}\">{}</a>\n\
         <span class=\"ctext\">{}{}</span>\n\
         </li>\n",
        depth.min(4),
        id,
        id,
        escape(&family),
        escape(&doc.name()),
        if retired { "1" } else { "0" },
        haystack,
        id,
        id,
        id,
        inline_markdown(&criterion.text),
        why,
    )
}

fn feature_section(doc: &Doc) -> String {
    let title = doc.title.clone().unwrap_or_else(|| doc.name());
    let mut out = String::new();

    out.push_str(&format!(
        "<section class=\"feature\" data-file=\"{}\">\n<h2>{}</h2>\n",
        escape(&doc.name()),
        escape(&title)
    ));

    let intent = paragraphs(&doc.intent);
    if !intent.is_empty() {
        out.push_str(&format!("<div class=\"intent\">{intent}</div>\n"));
    }

    out.push_str("<ul class=\"criteria\">\n");
    if doc.criteria.is_empty() && doc.retired.is_empty() {
        out.push_str("<li class=\"empty\">Nothing written down yet.</li>\n");
    }
    for criterion in &doc.criteria {
        out.push_str(&row(doc, criterion, false));
    }
    for criterion in &doc.retired {
        out.push_str(&row(doc, criterion, true));
    }
    out.push_str("</ul>\n</section>\n");
    out
}

const STYLE: &str = include_str!("view/style.css");
const SCRIPT: &str = include_str!("view/app.js");
// Copied from the CorvidLabs design system rather than hand-rolled, which its
// ADOPTION.md asks for by name. Inlined, because the page fetches nothing
// (hi: VIEW-2).
const THEME: &str = include_str!("view/theme.js");
const PREPAINT: &str = include_str!("view/prepaint.html");
const TOGGLE: &str = include_str!("view/toggle.html");

/// Build the whole page.
pub fn render(workspace: &Workspace, product_intent: Option<&str>, name: Option<&str>) -> String {
    // The name the author gave their product beats the directory it happens to
    // sit in (hi: VIEW-11).
    let title = name
        .map(str::to_string)
        .or_else(|| {
            workspace
                .root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "Intent".to_string());

    let total = workspace.criteria_count();
    let retired_total: usize = workspace.docs.iter().map(|d| d.retired.len()).sum();

    // One entry per feature, carrying its own count so a reader knows how big a
    // section is before going there (hi: VIEW-12.a).
    let nav_items: String = workspace
        .docs
        .iter()
        .map(|d| {
            format!(
                "<button type=\"button\" class=\"navitem\" data-value=\"{}\">\
                 <span class=\"navname\">{}</span>\n\
                 <span class=\"navcount\">{}</span>\n</button>",
                escape(&d.name()),
                escape(&d.title.clone().unwrap_or_else(|| d.name())),
                d.criteria.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

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

    let mut page = String::with_capacity(96 * 1024);
    page.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    page.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    page.push_str(&format!(
        "<title>{}: what this should be</title>\n",
        escape(&title)
    ));
    // Before any CSS, so a stored theme never flashes the other one first.
    page.push_str(PREPAINT);
    page.push_str("<style>\n");
    page.push_str(STYLE);
    page.push_str("</style>\n</head>\n<body>\n<div class=\"app\">\n");

    // The rail: the product's name, the search box, and every feature. It is
    // sticky, so navigation never scrolls away (hi: VIEW-12). With scripting
    // off the search box and the sort control are hidden, because neither can
    // do anything (hi: VIEW-10).
    page.push_str("<aside class=\"rail\">\n<div class=\"railstick\">\n<div class=\"railtop\">\n");
    page.push_str("<div class=\"brandrow\">\n");
    page.push_str(&format!("<p class=\"brand\">{}</p>\n", escape(&title)));
    page.push_str(TOGGLE);
    page.push_str("</div>\n");
    page.push_str(
        "<div class=\"searchwrap\" id=\"searchwrap\" hidden>\n\
         <input type=\"search\" id=\"q\" placeholder=\"Search\" \
         autocomplete=\"off\" spellcheck=\"false\" aria-label=\"Search criteria and ids\">\n\
         <kbd class=\"slash\">/</kbd>\n\
         </div>\n</div>\n",
    );
    page.push_str(&format!(
        "<nav class=\"nav\" id=\"nav\" aria-label=\"Features\">\n\
         <button type=\"button\" class=\"navitem\" data-value=\"\">\
         <span class=\"navname\">All criteria</span>\n\
         <span class=\"navcount\">{total}</span>\n</button>\n{nav_items}\n</nav>\n</div>\n"
    ));
    page.push_str(&format!(
        "<div class=\"railfoot\" id=\"railfoot\" hidden>\n\
         <label class=\"field\"><span>Order</span>\n\
         <select id=\"sort\" aria-label=\"Sort\">\n\
         <option value=\"doc\">Grouped by feature</option>\n\
         <option value=\"id\">By id</option>\n\
         <option value=\"family\">By family</option>\n\
         </select></label>\n\
         <label class=\"toggle\"><input type=\"checkbox\" id=\"showretired\"> \
         Show retired ({retired_total})</label>\n\
         <p class=\"count\" id=\"count\"></p>\n\
         <button type=\"button\" class=\"clear\" id=\"clear\" hidden>Reset</button>\n\
         </div>\n</aside>\n"
    ));

    // The document. The author's own prose comes first and nothing generic sits
    // above it (hi: VIEW-1.a, VIEW-16).
    page.push_str("<div class=\"doc\">\n");
    page.push_str(&lead);
    page.push_str("\n<main id=\"features\">\n");
    page.push_str(&features);
    page.push_str("</main>\n");
    page.push_str("<ul class=\"criteria flat\" id=\"flat\" hidden></ul>\n");
    page.push_str(
        "<p class=\"nomatch\" id=\"nomatch\" hidden>Nothing matches that. \
         <button type=\"button\" class=\"clear\" id=\"clear2\">Reset</button></p>\n",
    );

    page.push_str(&format!(
        "<footer>\n\
         <p>{total} criteria, {retired_total} retired. Every line is something a person asked for, \
         written as what this should be rather than as a report of what it currently does. \
         These are directions, and directions stay correct through a wrong turn.</p>\n\
         <p class=\"keys\">Click an id to copy its link. \
         <kbd>/</kbd> search, <kbd>j</kbd> <kbd>k</kbd> move, <kbd>Enter</kbd> copy, \
         <kbd>Esc</kbd> reset. Generated by <code>hi view</code>.</p>\n</footer>\n"
    ));

    page.push_str("</div>\n</div>\n<div class=\"toast\" id=\"toast\" hidden></div>\n<script>\n");
    page.push_str(THEME);
    page.push_str(SCRIPT);
    page.push_str("</script>\n</body>\n</html>\n");
    page
}

/// Write the page, defaulting to `intent.html` at the repository root.
pub fn write(workspace: &Workspace, out: Option<&str>) -> Result<String> {
    let raw = fs::read_to_string(workspace.intent_path()).ok();
    let name = raw.as_deref().and_then(product_name);
    let product_intent = raw.map(strip_index);
    let html = render(workspace, product_intent.as_deref(), name.as_deref());

    let path = match out {
        Some(out) => workspace.root.join(out),
        None => workspace.root.join("intent.html"),
    };
    fs::write(&path, html).with_context(|| format!("writing {}", path.display()))?;
    Ok(workspace.rel(&path))
}

/// The product's name, as the author wrote it at the top of `INTENT.md`.
/// `strip_index` drops that heading from the prose, so the page would otherwise
/// throw away the one place a person actually named their product and fall back
/// to a directory name (hi: VIEW-11).
fn product_name(raw: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
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

    const CHAT: &str = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nIt should feel like texting.\n\n## Criteria\n\n- **SEND-1**  I hit enter and it shows up.\n  - **SEND-1.a**  If I have no connection it queues.\n";

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
        assert_eq!(strip_comments("keep <!-- rest is gone"), "keep ");
    }

    #[test]
    fn the_page_fetches_nothing_from_the_network() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        // An inline script is fine. A request to somebody else's server is not,
        // because the page has to work offline and be sendable as one file.
        for probe in [
            "http://", "https://", "//fonts.", "<iframe", "@import", "<link", "src=",
        ] {
            assert!(!html.contains(probe), "the page must not reference {probe}");
        }
    }

    #[test]
    fn every_criterion_is_addressable_by_its_id() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        assert!(
            html.contains("id=\"SEND-1\""),
            "a row carries its id as an anchor"
        );
        assert!(
            html.contains("href=\"#SEND-1\""),
            "and the id links to itself"
        );
        assert!(html.contains("id=\"SEND-1.a\""));
        assert!(html.contains("href=\"#SEND-1.a\""));
    }

    #[test]
    fn rows_carry_what_the_controls_filter_on() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        assert!(html.contains("data-family=\"SEND\""));
        assert!(html.contains("data-file=\"chat\""));
        assert!(html.contains("data-retired=\"0\""));
        assert!(
            html.contains("data-find=\"send-1 i hit enter and it shows up. send\""),
            "the search haystack is lowercased and covers id, sentence and family"
        );
    }

    #[test]
    fn the_controls_offer_search_sort_and_filter() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        assert!(html.contains("<input type=\"search\" id=\"q\""));
        assert!(html.contains("<option value=\"id\">By id</option>"));
        assert!(html.contains("<option value=\"family\">By family</option>"));
        assert!(html.contains("class=\"navitem\" data-value=\"chat\""));
        assert!(html.contains("id=\"showretired\""));
    }

    #[test]
    fn hiding_a_row_actually_hides_it() {
        // `.row` sets `display: flex`, and an author rule outranks the user
        // agent's `[hidden] { display: none }` however specific it is. Without
        // the override the filters update the count and change nothing a
        // reader can see (hi: VIEW-6, VIEW-7).
        assert!(
            STYLE.contains("[hidden] { display: none !important; }"),
            "the stylesheet has to neutralize its own display rules for [hidden]"
        );
    }

    #[test]
    fn the_controls_are_hidden_until_the_script_runs() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        assert!(
            html.contains("id=\"searchwrap\" hidden>") && html.contains("id=\"railfoot\" hidden>"),
            "a reader without scripting must not see a search box that cannot search"
        );
        assert!(
            html.contains("<nav class=\"nav\" id=\"nav\""),
            "the feature list needs no script, so it is never hidden"
        );
    }

    #[test]
    fn copied_text_keeps_the_id_and_sentence_apart() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None, None);
        assert!(
            !html.contains("</a><span") && !html.contains("</span><span"),
            "no two inline elements may sit adjacent with no whitespace"
        );
    }

    #[test]
    fn page_carries_intent_prose_and_every_criterion() {
        let html = render(
            &workspace(&[("chat.md", CHAT)]),
            Some("The product why."),
            None,
        );
        assert!(html.contains("It should feel like texting."));
        assert!(html.contains("The product why."));
        assert!(html.contains("SEND-1.a"));
        assert!(html.contains("<h2>Chat</h2>"));
        assert!(html.contains("class=\"row d2\""));
        assert!(html.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn retired_criteria_are_present_and_marked() {
        let chat = "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  Live.\n\n## Retired\n\n- **SEND-2**  Gone.\n  retired: different product\n";
        let html = render(&workspace(&[("chat.md", chat)]), None, None);
        assert!(html.contains("data-retired=\"1\""));
        assert!(html.contains("SEND-2"));
        assert!(
            html.contains("different product"),
            "the reason travels with it"
        );
        assert!(html.contains("Show retired (1)"));
    }

    #[test]
    fn a_feature_whose_prose_is_only_the_template_comment_renders_no_empty_block() {
        let raw = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Billing\n\n## Intent\n\n<!-- What is this for? -->\n\n## Criteria\n\n- **SEND-1**  One.\n";
        let html = render(&workspace(&[("billing.md", raw)]), None, None);
        assert!(!html.contains("What is this for"));
        assert!(!html.contains("<div class=\"intent\"></div>"));
    }

    #[test]
    fn strips_the_generated_index_from_product_prose() {
        let raw = "# Product\n\nThe why.\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md)\n<!-- /hi:index -->\n";
        assert_eq!(strip_index(raw.to_string()), "The why.");
    }
}
