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

    let role = criterion.role.clone().unwrap_or_default();
    let sentence = if criterion.role.is_some() {
        crate::doc::without_role(&criterion.text)
    } else {
        criterion.text.clone()
    };

    // A lowercased haystack, so searching never has to walk the DOM.
    let haystack =
        escape(&format!("{} {} {} {}", criterion.raw_id, role, sentence, family).to_lowercase());

    let role_chip = if role.is_empty() {
        String::new()
    } else {
        format!("<span class=\"role\">{}</span>\n", escape(&role))
    };
    let why = criterion
        .note
        .as_ref()
        .map(|n| format!("\n<span class=\"why\">{}</span>", inline_markdown(n)))
        .unwrap_or_default();

    format!(
        "<li class=\"row d{}\" id=\"{}\" data-id=\"{}\" data-role=\"{}\" data-family=\"{}\" \
         data-file=\"{}\" data-retired=\"{}\" data-find=\"{}\">\n\
         <a class=\"cid\" href=\"#{}\" title=\"Link to {}\">{}</a>\n\
         <span class=\"ctext\">{}{}{}</span>\n\
         </li>\n",
        depth.min(4),
        id,
        id,
        escape(&role),
        escape(&family),
        escape(&doc.name()),
        if retired { "1" } else { "0" },
        haystack,
        id,
        id,
        id,
        role_chip,
        inline_markdown(&sentence),
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

/// Build the whole page.
pub fn render(workspace: &Workspace, product_intent: Option<&str>) -> String {
    let title = workspace
        .root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Intent".to_string());

    let total = workspace.criteria_count();
    let retired_total: usize = workspace.docs.iter().map(|d| d.retired.len()).sum();

    let mut roles: Vec<String> = Vec::new();
    for doc in &workspace.docs {
        for criterion in doc.criteria.iter().chain(doc.retired.iter()) {
            if let Some(role) = &criterion.role
                && !roles.contains(role)
            {
                roles.push(role.clone());
            }
        }
    }
    roles.sort();

    let role_chips: String = roles
        .iter()
        .map(|r| {
            format!(
                "<button type=\"button\" class=\"chip\" data-filter=\"role\" data-value=\"{}\">{}</button>",
                escape(r),
                escape(r)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let file_chips: String = workspace
        .docs
        .iter()
        .map(|d| {
            format!(
                "<button type=\"button\" class=\"chip\" data-filter=\"file\" data-value=\"{}\">{}</button>",
                escape(&d.name()),
                escape(&d.title.clone().unwrap_or_else(|| d.name()))
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
    page.push_str("<style>\n");
    page.push_str(STYLE);
    page.push_str("</style>\n</head>\n<body>\n<div class=\"wrap\">\n");

    page.push_str("<header>\n<p class=\"eyebrow\">What this should be</p>\n");
    page.push_str(&format!("<h1>{}</h1>\n", escape(&title)));
    page.push_str(&lead);
    page.push_str("\n<hr class=\"rule\">\n</header>\n");

    // Hidden until the script runs, so nobody is shown a search box that
    // cannot search.
    page.push_str("<div class=\"controls\" id=\"controls\" hidden>\n");
    page.push_str(
        "<div class=\"searchrow\">\n\
         <input type=\"search\" id=\"q\" placeholder=\"Search criteria, ids, roles. Press /\" \
         autocomplete=\"off\" spellcheck=\"false\">\n\
         <select id=\"sort\" aria-label=\"Sort\">\n\
         <option value=\"doc\">Grouped by feature</option>\n\
         <option value=\"id\">Sort by id</option>\n\
         <option value=\"role\">Sort by role</option>\n\
         <option value=\"family\">Sort by family</option>\n\
         </select>\n\
         </div>\n",
    );
    if !roles.is_empty() {
        page.push_str(&format!(
            "<div class=\"chips\" id=\"rolechips\">\n{role_chips}\n</div>\n"
        ));
    }
    page.push_str(&format!(
        "<div class=\"chips\" id=\"filechips\">\n{file_chips}\n</div>\n"
    ));
    page.push_str(&format!(
        "<div class=\"statusrow\">\n\
         <span class=\"count\" id=\"count\"></span>\n\
         <label class=\"toggle\"><input type=\"checkbox\" id=\"showretired\"> \
         Show retired ({retired_total})</label>\n\
         <button type=\"button\" class=\"clear\" id=\"clear\" hidden>Clear filters</button>\n\
         </div>\n</div>\n"
    ));

    page.push_str("<main id=\"features\">\n");
    page.push_str(&features);
    page.push_str("</main>\n");
    page.push_str("<ul class=\"criteria flat\" id=\"flat\" hidden></ul>\n");
    page.push_str("<p class=\"nomatch\" id=\"nomatch\" hidden>Nothing matches that.</p>\n");

    page.push_str(&format!(
        "<footer>\n\
         <p>{total} criteria, {retired_total} retired. Every line is something a person asked for, \
         written as what this should be rather than as a report of what it currently does. A line \
         can describe something true today, something a year out, or something since revised. \
         These are directions, and directions stay correct through a wrong turn.</p>\n\
         <p>Click any id to link straight to it. Press / to search. Generated by \
         <code>hi view</code>.</p>\n</footer>\n"
    ));

    page.push_str("</div>\n<script>\n");
    page.push_str(SCRIPT);
    page.push_str("</script>\n</body>\n</html>\n");
    page
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

    const CHAT: &str = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nIt should feel like texting.\n\n## Criteria\n\n- **SEND-1**  As a member, I hit enter and it shows up.\n  - **SEND-1.a**  As a member, if I have no connection it queues.\n";

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
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
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
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
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
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
        assert!(html.contains("data-role=\"member\""));
        assert!(html.contains("data-family=\"SEND\""));
        assert!(html.contains("data-file=\"chat\""));
        assert!(html.contains("data-retired=\"0\""));
        assert!(
            html.contains("data-find=\"send-1 member i hit enter and it shows up. send\""),
            "the search haystack is lowercased and covers id, role, sentence and family"
        );
    }

    #[test]
    fn the_controls_offer_search_sort_and_filter() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
        assert!(html.contains("<input type=\"search\" id=\"q\""));
        assert!(html.contains("<option value=\"id\">Sort by id</option>"));
        assert!(html.contains("<option value=\"role\">Sort by role</option>"));
        assert!(html.contains("<option value=\"family\">Sort by family</option>"));
        assert!(html.contains("data-filter=\"role\" data-value=\"member\""));
        assert!(html.contains("data-filter=\"file\" data-value=\"chat\""));
        assert!(html.contains("id=\"showretired\""));
    }

    #[test]
    fn the_controls_are_hidden_until_the_script_runs() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
        assert!(
            html.contains("<div class=\"controls\" id=\"controls\" hidden>"),
            "a reader without scripting must not see a search box that cannot search"
        );
    }

    #[test]
    fn the_role_label_is_styled_not_merely_emitted() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
        assert!(html.contains("class=\"role\""), "the page labels the voice");
        assert!(
            html.contains(".role {"),
            "and the label has a rule behind it, or it renders as bare text"
        );
    }

    #[test]
    fn copied_text_keeps_the_id_and_sentence_apart() {
        let html = render(&workspace(&[("chat.md", CHAT)]), None);
        assert!(
            !html.contains("</a><span") && !html.contains("</span><span"),
            "no two inline elements may sit adjacent with no whitespace"
        );
    }

    #[test]
    fn page_carries_intent_prose_and_every_criterion() {
        let html = render(&workspace(&[("chat.md", CHAT)]), Some("The product why."));
        assert!(html.contains("It should feel like texting."));
        assert!(html.contains("The product why."));
        assert!(html.contains("SEND-1.a"));
        assert!(html.contains("<h2>Chat</h2>"));
        assert!(html.contains("class=\"row d2\""));
        assert!(html.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn retired_criteria_are_present_and_marked() {
        let chat = "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  As a member, live.\n\n## Retired\n\n- **SEND-2**  As a member, gone.\n  retired: different product\n";
        let html = render(&workspace(&[("chat.md", chat)]), None);
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
        let raw = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Billing\n\n## Intent\n\n<!-- What is this for? -->\n\n## Criteria\n\n- **SEND-1**  As a member, one.\n";
        let html = render(&workspace(&[("billing.md", raw)]), None);
        assert!(!html.contains("What is this for"));
        assert!(!html.contains("<div class=\"intent\"></div>"));
    }

    #[test]
    fn strips_the_generated_index_from_product_prose() {
        let raw = "# Product\n\nThe why.\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md)\n<!-- /hi:index -->\n";
        assert_eq!(strip_index(raw.to_string()), "The why.");
    }
}
