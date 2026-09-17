//! Reading and writing `hi/*.md`.
//!
//! A hi file is markdown a human could have typed by hand. The parser keeps the
//! original lines so edits are surgical: capture splices new lines in and never
//! reformats prose it did not write.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::id::{Id, IdError, looks_like_id};

/// Where a criterion lives in its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Criteria,
    Retired,
}

/// One criterion: an id, a human sentence, and where it sits on disk.
#[derive(Debug, Clone)]
pub struct Criterion {
    /// Parsed id, or `None` when the token was id-shaped but malformed.
    pub id: Option<Id>,
    /// The id exactly as written, for error messages.
    pub raw_id: String,
    /// Why the id failed to parse, when it did.
    pub id_error: Option<IdError>,
    /// The sentence, with continuation lines joined by a single space.
    pub text: String,
    /// A `retired:` note, when present.
    pub note: Option<String>,
    /// 0-based index of the line the id sits on.
    pub line: usize,
    /// 0-based index of this criterion's last line, inclusive.
    pub end_line: usize,
    pub section: Section,
}

impl Criterion {
    /// The 1-based line number, for humans.
    pub fn line_no(&self) -> usize {
        self.line + 1
    }
}

/// What a write said it was about to do, so the reparse can hold it to that.
struct Promise<'a> {
    /// The verb, for the first words of a refusal.
    verb: &'a str,
    /// The ids the write is about, as they are written in the file.
    ids: Vec<String>,
    /// Where every one of them has to be readable once the buffer is parsed.
    section: Section,
    /// True when those ids are newly written, false when they only moved.
    added: bool,
}

/// The first of `expected` that `found` does not account for, as a list.
///
/// A multiset difference rather than a set one: a file may legitimately hold
/// the same id twice, and `check` is what reports that, so losing one of the
/// two is still a loss.
fn missing(expected: &[String], found: &[String]) -> Option<String> {
    let mut rest: Vec<&String> = found.iter().collect();
    let mut lost: Vec<&str> = Vec::new();
    for id in expected {
        match rest.iter().position(|other| *other == id) {
            Some(at) => {
                rest.remove(at);
            }
            None => lost.push(id),
        }
    }
    (!lost.is_empty()).then(|| lost.join(", "))
}

/// Frontmatter, hand-parsed so a hi file needs no YAML library.
#[derive(Debug, Clone, Default)]
pub struct Front {
    pub version: Option<u32>,
    pub families: Vec<String>,
    pub owner: Option<String>,
    /// Line range of the frontmatter block, inclusive of both `---` fences.
    pub range: Option<(usize, usize)>,
    /// Inclusive line span of the `families` entry, including block items.
    pub families_span: Option<(usize, usize)>,
    /// True when `families` was written as a YAML block list rather than inline.
    pub families_block: bool,
}

/// One parsed `hi/*.md`.
#[derive(Debug, Clone)]
pub struct Doc {
    pub path: PathBuf,
    pub front: Front,
    pub title: Option<String>,
    pub intent: String,
    pub criteria: Vec<Criterion>,
    pub retired: Vec<Criterion>,
    /// Every line of the file, kept for surgical edits.
    pub lines: Vec<String>,
    /// Line index just past the end of `## Criteria`, where new criteria append.
    criteria_end: Option<usize>,
    /// Line index of the `## Criteria` heading itself.
    criteria_heading: Option<usize>,
    /// Criterion-shaped lines found outside any section, as (line index, token).
    /// Nothing reads these as criteria, so `check` reports them rather than
    /// letting a criterion disappear because a heading moved above it.
    pub stray: Vec<(usize, String)>,
    /// True when the file ended with a newline, so writing round-trips.
    trailing_newline: bool,
    /// The line ending the file already uses, so hi does not convert it.
    newline: &'static str,
}

impl Doc {
    /// Read and parse one file.
    pub fn load(path: &Path) -> Result<Doc> {
        let raw =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Ok(Doc::parse(path.to_path_buf(), &raw))
    }

    /// Parse already-loaded text. Split out so tests need no filesystem.
    pub fn parse(path: PathBuf, raw: &str) -> Doc {
        // An editor-written BOM would sit before the `---` and hide the
        // frontmatter from every check that follows.
        let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
        let trailing_newline = raw.ends_with('\n');
        // `lines()` drops the `\r`, so remember which ending to write back.
        let crlf = raw.matches("\r\n").count();
        let lf = raw.matches('\n').count() - crlf;
        let newline = if crlf > lf { "\r\n" } else { "\n" };
        let lines: Vec<String> = raw.lines().map(|l| l.to_string()).collect();

        let mut doc = Doc {
            path,
            front: Front::default(),
            title: None,
            intent: String::new(),
            criteria: Vec::new(),
            retired: Vec::new(),
            lines,
            criteria_end: None,
            criteria_heading: None,
            stray: Vec::new(),
            trailing_newline,
            newline,
        };

        let start = doc.parse_front();
        doc.parse_body(start);
        doc
    }

    /// Parse the frontmatter block and return the line the body starts on.
    fn parse_front(&mut self) -> usize {
        if self.lines.first().map(|l| l.trim_end()) != Some("---") {
            return 0;
        }
        let Some(close) = (1..self.lines.len()).find(|&i| self.lines[i].trim_end() == "---") else {
            // An unterminated fence is not frontmatter; treat it all as body.
            return 0;
        };

        let mut index = 1;
        while index < close {
            let line = self.lines[index].clone();
            let Some((key, value)) = line.split_once(':') else {
                index += 1;
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "hi" => self.front.version = value.parse::<u32>().ok(),
                "families" | "family" => {
                    let start = index;
                    if value.is_empty() {
                        // Block form:
                        //   families:
                        //     - SEND
                        //     - RECEIPT
                        let mut end = index;
                        let mut item = index + 1;
                        while item < close {
                            let candidate = self.lines[item].trim();
                            let indented = self.lines[item].starts_with(' ')
                                || self.lines[item].starts_with('\t');
                            match candidate.strip_prefix('-') {
                                Some(name) if indented => {
                                    let name = name.trim().trim_matches(['"', '\'']);
                                    if !name.is_empty() {
                                        self.front.families.push(name.to_string());
                                    }
                                    end = item;
                                    item += 1;
                                }
                                _ => break,
                            }
                        }
                        self.front.families_block = true;
                        self.front.families_span = Some((start, end));
                        index = end + 1;
                        continue;
                    }
                    // Inline form: `families: [SEND, RECEIPT]` or `family: SEND`.
                    let inner = value.trim_start_matches('[').trim_end_matches(']');
                    for name in inner.split(',') {
                        let name = name.trim().trim_matches(['"', '\'']);
                        if !name.is_empty() {
                            self.front.families.push(name.to_string());
                        }
                    }
                    self.front.families_span = Some((start, start));
                }
                "owner" if !value.is_empty() => {
                    self.front.owner = Some(value.trim_matches(['"', '\'']).to_string())
                }
                _ => {}
            }
            index += 1;
        }

        self.front.range = Some((0, close));
        close + 1
    }

    fn parse_body(&mut self, start: usize) {
        let mut section: Option<Section> = None;
        let mut intent_lines: Vec<String> = Vec::new();
        let mut in_intent = false;
        let mut index = start;

        // A fenced code block is opaque. Without this, documenting the format
        // inside your own `## Intent` turns the example into real criteria.
        // The map is built once, by the same function every write path asks
        // where the sections are, so the two can no longer disagree
        // (hi: FILE-22, DECISIONS.md §31).
        let fences = fence_map(&self.lines, start);

        while index < self.lines.len() {
            let line = self.lines[index].clone();
            let trimmed = line.trim();

            if fences.inside[index] {
                // A fence inside `## Intent` is somebody documenting the format
                // in their own prose, and stays opaque (hi: FILE-9). A fence
                // inside `## Criteria` or `## Retired` hides a criterion from
                // every one of the six checks, and capture would then hand the
                // same id out again: two `- **SEND-2**` lines, different
                // sentences, `hi check` clean. Record it as stray, which is
                // exactly what it is: a criterion nothing reads where it sits
                // (hi: FILE-20).
                if in_intent {
                    intent_lines.push(line);
                } else if !trimmed.is_empty() && is_criterion_line(trimmed) {
                    let first = strip_bullet(trimmed)
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    self.stray.push((index, first));
                }
                index += 1;
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("# ")
                && !trimmed.starts_with("##")
            {
                if self.title.is_none() {
                    self.title = Some(rest.trim().to_string());
                }
                // A level-one heading closes an open section just as `## ` does.
                // Without this the append point is lost and a new family lands
                // at the top of the criteria block instead of the bottom.
                if section == Some(Section::Criteria) {
                    self.criteria_end = Some(last_content_line(&self.lines, index) + 1);
                }
                section = None;
                in_intent = false;
                index += 1;
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("## ") {
                // Leaving `## Criteria` fixes where new criteria append.
                if section == Some(Section::Criteria) {
                    self.criteria_end = Some(last_content_line(&self.lines, index) + 1);
                }
                let name = rest.trim().to_ascii_lowercase();
                in_intent = name == "intent";
                section = match name.as_str() {
                    "criteria" => {
                        self.criteria_heading = Some(index);
                        Some(Section::Criteria)
                    }
                    "retired" => Some(Section::Retired),
                    _ => None,
                };
                index += 1;
                continue;
            }

            if in_intent {
                intent_lines.push(line);
                index += 1;
                continue;
            }

            let Some(current) = section else {
                // Outside any section a criterion is invisible. Remember it so
                // `check` can say so instead of the line silently vanishing.
                if !trimmed.is_empty() && is_criterion_line(trimmed) {
                    let first = strip_bullet(trimmed)
                        .split_whitespace()
                        .next()
                        .unwrap_or("");
                    self.stray.push((index, first.to_string()));
                }
                index += 1;
                continue;
            };

            // A criterion is an optional bullet then an id, at any indent.
            if trimmed.is_empty() || !is_criterion_line(trimmed) {
                index += 1;
                continue;
            }

            let (criterion, next) = self.read_criterion(index, current);
            match current {
                Section::Criteria => self.criteria.push(criterion),
                Section::Retired => self.retired.push(criterion),
            }
            index = next;
        }

        if section == Some(Section::Criteria) && self.criteria_end.is_none() {
            self.criteria_end = Some(last_content_line(&self.lines, self.lines.len()) + 1);
        }

        self.intent = intent_lines.join("\n").trim().to_string();
    }

    /// Read one criterion starting at `start`, returning it and the next line.
    fn read_criterion(&self, start: usize, section: Section) -> (Criterion, usize) {
        let line = &self.lines[start];
        let content = strip_bullet(line.trim());
        let (written_id, rest) = content
            .split_once(char::is_whitespace)
            .unwrap_or((content, ""));
        // `raw_id` is the id as it means, not as it was decorated.
        let raw_id = strip_emphasis(written_id);

        let mut text = rest.trim().to_string();
        let mut note: Option<String> = None;
        let mut end = start;
        let mut index = start + 1;

        // Continuations are indented, non-blank, and unbroken by a blank line.
        while index < self.lines.len() {
            let next = &self.lines[index];
            if next.trim().is_empty() || !(next.starts_with(' ') || next.starts_with('\t')) {
                break;
            }
            // An indented line that is itself a criterion ends this one.
            if is_criterion_line(next.trim()) {
                break;
            }
            let content = next.trim();
            if let Some(reason) = content.strip_prefix("retired:") {
                note = Some(reason.trim().to_string());
            } else if !text.is_empty() {
                text.push(' ');
                text.push_str(content);
            } else {
                text.push_str(content);
            }
            end = index;
            index += 1;
        }

        let (id, id_error) = match Id::parse(raw_id) {
            Ok(id) => (Some(id), None),
            Err(err) => (None, Some(err)),
        };

        (
            Criterion {
                id,
                raw_id: raw_id.to_string(),
                id_error,
                text,
                note,
                line: start,
                end_line: end,
                section,
            },
            index,
        )
    }

    /// Every criterion in the file, active and retired.
    pub fn all(&self) -> impl Iterator<Item = &Criterion> {
        self.criteria.iter().chain(self.retired.iter())
    }

    /// Families actually used by criteria, in first-seen order.
    pub fn used_families(&self) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for criterion in self.all() {
            if let Some(id) = &criterion.id
                && !seen.contains(&id.family)
            {
                seen.push(id.family.clone());
            }
        }
        seen
    }

    /// Every id this file makes findable again, as written, sorted.
    ///
    /// Readable is the whole of it: an id under `## Criteria` or `## Retired`
    /// is one hi can find again; an id anywhere else is one it cannot, however
    /// plainly it sits there on the page.
    fn readable(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.all().map(|c| c.raw_id.clone()).collect();
        ids.sort();
        ids
    }

    /// Read a buffer back before hi makes it its own, and refuse it unless the
    /// edit really happened where the verb said it did.
    ///
    /// hi's one promise is that an id is permanent (DECISIONS.md §26), and the
    /// two remaining ways to break it were the same mistake twice: a write
    /// deciding where a section is without asking the parser. So no write site
    /// is trusted to know where it landed. Each one parses what it is about to
    /// save and holds it to three things: the ids the verb named are readable
    /// in the section it named, every id the file already made readable still
    /// is, and no line has been stranded that was not stranded before. A write
    /// that fails any of them is refused with the file untouched, which beats
    /// reporting success into a place nothing reads (hi: FILE-22, CAPTURE-5).
    fn read_back(&self, proposed: &str, promise: Promise<'_>) -> Result<Doc> {
        let after = Doc::parse(self.path.clone(), proposed);
        let file = self.path.display();

        // An unfinished fence is the likeliest reason a write lands nowhere, and
        // saying so is the difference between a refusal somebody can act on and
        // one they can only be annoyed by.
        let hint = match fence_map(&after.lines, after.body_start()).unclosed {
            Some((marker, line)) => format!(
                "\nhint:  an unclosed {} fence opens on line {}, so everything below it is an \
                 example rather than part of the file. Close it, then try again",
                marker.to_string().repeat(3),
                line + 1
            ),
            None => String::new(),
        };

        // What the verb said it was doing, first: the id has to be readable in
        // the section the verb named, not merely somewhere in the file.
        let landed = match promise.section {
            Section::Criteria => &after.criteria,
            Section::Retired => &after.retired,
        };
        if let Some(adrift) = promise
            .ids
            .iter()
            .find(|want| !landed.iter().any(|c| &&c.raw_id == want))
        {
            bail!(
                "{} {adrift} would put it in {file} where hi cannot read it back, \
                 so nothing was written{hint}",
                promise.verb
            );
        }

        // Then the collateral: an edit that lands correctly and takes somebody
        // else's criterion with it is the same failure one line over.
        let mut expected = self.readable();
        if promise.added {
            expected.extend(promise.ids.iter().cloned());
            expected.sort();
        }
        if let Some(lost) = missing(&expected, &after.readable()) {
            bail!(
                "{} {} would leave {lost} unreadable in {file}, and an id is permanent, \
                 so nothing was written{hint}",
                promise.verb,
                promise.ids.join(", ")
            );
        }

        if after.stray.len() > self.stray.len() {
            bail!(
                "{} {} would strand a criterion in {file} where nothing reads it, \
                 so nothing was written{hint}",
                promise.verb,
                promise.ids.join(", ")
            );
        }

        Ok(after)
    }

    /// Insert a new criterion, keeping each family's block contiguous and each
    /// parent immediately followed by its own descendants.
    pub fn insert(&mut self, id: &Id, text: &str) -> Result<()> {
        // Every refusal below has to leave the caller's document exactly as it
        // found it, because `capture` holds one Doc for the whole run and a
        // half-spliced buffer would be the next thing saved (hi: CAPTURE-5).
        let before = self.clone();
        let spliced = self.insert_inner(id, text).and_then(|()| {
            before.read_back(
                &self.to_text(),
                Promise {
                    verb: "capturing",
                    ids: vec![id.to_string()],
                    section: Section::Criteria,
                    added: true,
                },
            )
        });
        match spliced {
            // The verified parse is deliberately thrown away rather than
            // adopted. `insert`'s index bookkeeping is the contract with
            // `rewrite_families` and with the next insert, and re-deriving it
            // here would make that contract untestable, so `capture` still
            // reloads the file it saved before anything counts
            // (DECISIONS.md §30, hi: INDEX-4).
            Ok(_) => Ok(()),
            Err(err) => {
                *self = before;
                Err(err)
            }
        }
    }

    fn insert_inner(&mut self, id: &Id, text: &str) -> Result<()> {
        // Without a section to land in, a criterion would be appended wherever
        // the file happens to end (inside the intent prose, or below a heading
        // where nothing would ever read it). Make the section instead.
        if self.criteria_heading.is_none() {
            // `+ 1` runs off the end of an empty file: a zero-byte `hi/*.md`
            // yields no lines at all, and splicing at 1 into a Vec of length 0
            // panics. An empty file is what `touch`, a crashed editor or a
            // partial checkout leaves behind, and it should get the same
            // "no frontmatter" refusal any other unusable file gets
            // (hi: CAPTURE-12).
            let at = (last_content_line(&self.lines, self.lines.len()) + 1).min(self.lines.len());
            let opening = vec![String::new(), "## Criteria".to_string(), String::new()];
            let count = opening.len();
            self.lines.splice(at..at, opening);
            self.shift_after(at, count as isize);
            self.criteria_heading = Some(at + 1);
        }

        let (at, blank_before) = self.insertion_point(id);
        let mut rendered = render_criterion(id, text);
        if blank_before {
            rendered.insert(0, String::new());
        }
        let count = rendered.len();
        self.lines.splice(at..at, rendered);

        // Everything at or after the insertion point shifts down.
        for criterion in self.criteria.iter_mut().chain(self.retired.iter_mut()) {
            if criterion.line >= at {
                criterion.line += count;
                criterion.end_line += count;
            }
        }
        if let Some(end) = self.criteria_end
            && end >= at
        {
            self.criteria_end = Some(end + count);
        }
        if let Some(heading) = self.criteria_heading
            && heading >= at
        {
            self.criteria_heading = Some(heading + count);
        }
        for (line, _) in self.stray.iter_mut() {
            if *line >= at {
                *line += count;
            }
        }

        // Declare the family if the frontmatter has not seen it yet.
        if !self.front.families.iter().any(|f| f == &id.family) {
            self.front.families.push(id.family.clone());
            self.rewrite_families()?;
        }

        // Deliberately no reparse here, unlike `retire` and
        // `set_retired_reason`. The shifting above is the contract with
        // `rewrite_families` and with the next insert, and re-deriving would
        // make it untestable. The cost is that `self.criteria` does not yet
        // hold the line just spliced in, so a caller that goes on to count
        // criteria has to reload the saved file first; `capture` does
        // (hi: INDEX-4).
        Ok(())
    }

    /// Where a new criterion's first line goes, and whether it needs a blank
    /// line above it. A new family starts its own block; everything else joins
    /// the block it belongs to.
    fn insertion_point(&self, id: &Id) -> (usize, bool) {
        // Beneath a parent: after the parent and all of its existing descendants.
        if let Some(parent) = id.parent() {
            let mut at: Option<usize> = None;
            for criterion in &self.criteria {
                let Some(other) = &criterion.id else { continue };
                if *other == parent || other.is_descendant_of(&parent) {
                    at = Some(criterion.end_line + 1);
                }
            }
            if let Some(at) = at {
                return (at, false);
            }
        }

        // Otherwise after the last criterion of the same family.
        let mut at: Option<usize> = None;
        for criterion in &self.criteria {
            if let Some(other) = &criterion.id
                && other.family == id.family
            {
                at = Some(criterion.end_line + 1);
            }
        }
        if let Some(at) = at {
            return (at, false);
        }

        // A family this file has not seen before opens a new block.
        if !self.criteria.is_empty()
            && let Some(end) = self.criteria_end
        {
            return (end, true);
        }

        // An empty section: sit below the heading, keeping its blank line.
        if let Some(heading) = self.criteria_heading {
            let mut at = heading + 1;
            if self.lines.get(at).is_some_and(|l| l.trim().is_empty()) {
                at += 1;
            }
            return (at, false);
        }

        (self.lines.len(), false)
    }

    /// Rewrite the `families:` line in place, or add one to the frontmatter.
    fn rewrite_families(&mut self) -> Result<()> {
        let Some((open, close)) = self.front.range else {
            bail!(
                "{} has no frontmatter, so add `---\\nhi: 1\\n---` at the top",
                self.path.display()
            )
        };

        // Keep the style the file already uses; reformatting someone's
        // frontmatter is exactly the kind of edit hi promises not to make.
        let replacement: Vec<String> = if self.front.families_block {
            let mut block = vec!["families:".to_string()];
            block.extend(self.front.families.iter().map(|f| format!("  - {f}")));
            block
        } else {
            vec![format!("families: [{}]", self.front.families.join(", "))]
        };

        let (start, end) = match self.front.families_span {
            Some(span) => span,
            // No families entry yet: add one just inside the closing fence.
            None => (close, close.wrapping_sub(1)),
        };

        let removed = if end >= start { end - start + 1 } else { 0 };
        let added = replacement.len();
        self.lines.splice(start..start + removed, replacement);

        let delta = added as isize - removed as isize;
        if delta != 0 {
            self.shift_after(start, delta);
        }
        self.front.range = Some((open, (close as isize + delta) as usize));
        self.front.families_span = Some((start, start + added - 1));
        Ok(())
    }

    /// Move every tracked position at or after `at` by `delta` lines.
    fn shift_after(&mut self, at: usize, delta: isize) {
        let bump = |value: &mut usize| {
            if *value >= at {
                *value = (*value as isize + delta) as usize;
            }
        };
        for criterion in self.criteria.iter_mut().chain(self.retired.iter_mut()) {
            bump(&mut criterion.line);
            bump(&mut criterion.end_line);
        }
        if let Some(mut end) = self.criteria_end {
            bump(&mut end);
            self.criteria_end = Some(end);
        }
        if let Some(mut heading) = self.criteria_heading {
            bump(&mut heading);
            self.criteria_heading = Some(heading);
        }
        for (line, _) in self.stray.iter_mut() {
            bump(line);
        }
    }

    /// Move a criterion and everything beneath it into `## Retired`.
    ///
    /// Cases go with their parent, because a case whose parent has been retired
    /// is an orphan that `check` would then report. The section is created when
    /// the file has none, and the id stays reserved forever either way.
    pub fn retire(&mut self, id: &Id, reason: Option<&str>) -> Result<Vec<String>> {
        let before = self.clone();
        match self.retire_inner(id, reason) {
            Ok(taken) => Ok(taken),
            Err(err) => {
                *self = before;
                Err(err)
            }
        }
    }

    fn retire_inner(&mut self, id: &Id, reason: Option<&str>) -> Result<Vec<String>> {
        let before = self.clone();
        let mut ranges: Vec<(usize, usize)> = self
            .criteria
            .iter()
            .filter(|c| {
                c.id.as_ref()
                    .is_some_and(|other| other == id || other.is_descendant_of(id))
            })
            .map(|c| (c.line, c.end_line))
            .collect();

        if ranges.is_empty() {
            bail!("{id} is not an active criterion in {}", self.path.display());
        }
        ranges.sort_unstable();
        // Name what went. A case can belong to a different concern than its
        // parent, and taking it along silently is a decision made for you.
        let mut taken: Vec<String> = self
            .criteria
            .iter()
            .filter(|c| {
                c.id.as_ref()
                    .is_some_and(|other| other.is_descendant_of(id))
            })
            .map(|c| c.raw_id.clone())
            .collect();
        taken.sort();

        let mut moved: Vec<String> = Vec::new();
        for (start, end) in &ranges {
            moved.extend_from_slice(&self.lines[*start..=*end]);
        }
        if let Some(reason) = reason.map(str::trim).filter(|r| !r.is_empty()) {
            // Directly under the criterion it explains. Putting it after the
            // whole block reads tidily and parses as a note on the LAST case,
            // so hi wrote files that failed its own check. As a markdown list
            // item's continuation line it also renders correctly, with any
            // nested cases following it.
            let indent = " ".repeat(moved[0].len() - moved[0].trim_start().len() + 2);
            moved.insert(1, format!("{indent}retired: {}", one_line(reason)));
        }

        // Highest first, so the earlier ranges keep their indexes.
        for (start, end) in ranges.iter().rev() {
            self.lines.drain(*start..=*end);
        }

        let at = self.retired_end();
        let mut block = Vec::new();
        if self.retired_heading().is_none() {
            block.push(String::new());
            block.push("## Retired".to_string());
            block.push(String::new());
        }
        block.extend(moved);
        self.lines.splice(at..at, block);

        // Positions moved in both directions, so re-derive them rather than
        // trying to patch each one — and, on the way, refuse a buffer where the
        // criterion did not actually arrive under `## Retired`. Retiring is the
        // verb that frees nothing, so a retirement that lands anywhere else is
        // the one promise breaking (hi: FILE-22, RETIRE-2).
        let mut ids = vec![id.to_string()];
        ids.extend(taken.iter().cloned());
        *self = before.read_back(
            &self.to_text(),
            Promise {
                verb: "retiring",
                ids,
                section: Section::Retired,
                added: false,
            },
        )?;
        Ok(taken)
    }

    /// Record why an already retired criterion was retired.
    ///
    /// Retiring in a hurry and explaining later is the normal shape of
    /// changing your mind, and without this the only way to add the reason was
    /// to hand-edit the file, which is the thing the verb exists to remove
    /// (hi: RETIRE-1.b).
    pub fn set_retired_reason(&mut self, id: &Id, reason: &str) -> Result<()> {
        let before = self.clone();
        match self.set_retired_reason_inner(id, reason) {
            Ok(()) => Ok(()),
            Err(err) => {
                *self = before;
                Err(err)
            }
        }
    }

    fn set_retired_reason_inner(&mut self, id: &Id, reason: &str) -> Result<()> {
        let before = self.clone();
        let Some(criterion) = self
            .retired
            .iter()
            .find(|c| c.id.as_ref() == Some(id))
            .cloned()
        else {
            bail!("{id} is not retired in {}", self.path.display());
        };

        let indent = {
            let line = &self.lines[criterion.line];
            " ".repeat(line.len() - line.trim_start().len() + 2)
        };
        let rendered = format!("{indent}retired: {}", one_line(reason));

        // Replace an existing note, or add one after the criterion's last line.
        let existing = (criterion.line..=criterion.end_line)
            .find(|i| self.lines[*i].trim_start().starts_with("retired:"));
        match existing {
            Some(at) => self.lines[at] = rendered,
            None => self.lines.insert(criterion.end_line + 1, rendered),
        }

        *self = before.read_back(
            &self.to_text(),
            Promise {
                verb: "recording why",
                ids: vec![id.to_string()],
                section: Section::Retired,
                added: false,
            },
        )?;
        Ok(())
    }

    /// The line the body starts on: just past the frontmatter, or line 0.
    ///
    /// The same number `parse_front` hands `parse_body`, so a fence counted
    /// here is a fence the parser counted too.
    fn body_start(&self) -> usize {
        self.front.range.map(|(_, close)| close + 1).unwrap_or(0)
    }

    /// Where `## Retired` is, as the *parser* sees it.
    ///
    /// This used to scan the raw lines. A `## Retired` inside a fenced example
    /// under `## Intent` is a heading to a raw scan and prose to the parser
    /// (hi: FILE-9), and the disagreement cost a criterion: `retire` dropped it
    /// into the intent prose, printed success, and handed the id back out
    /// (hi: FILE-22, RETIRE-5).
    fn retired_heading(&self) -> Option<usize> {
        let fences = fence_map(&self.lines, self.body_start());
        (self.body_start()..self.lines.len()).find(|&index| {
            !fences.inside[index] && self.lines[index].trim().eq_ignore_ascii_case("## Retired")
        })
    }

    /// Where a newly retired criterion should be appended.
    ///
    /// Inside `## Retired`, which is not always the end of the file. Both arms
    /// of this used to append at EOF, so retiring into a file whose `## Retired`
    /// was followed by any other section printed "retired" and put the
    /// criterion under that other section instead, outside every section hi
    /// reads. From there `hi ls --retired` lost it and the id could be captured
    /// again, which is the one thing hi promises cannot happen
    /// (hi: RETIRE-5, FILE-13).
    fn retired_end(&self) -> usize {
        match self.retired_heading() {
            // The last line with content before the next heading of any level,
            // so the criterion lands at the bottom of the retired block rather
            // than at the bottom of the file.
            Some(at) => {
                // Fenced, so a `#` inside an example under `## Retired` does
                // not cut the block short (hi: FILE-9, FILE-22).
                let fences = fence_map(&self.lines, self.body_start());
                let next = (at + 1..self.lines.len())
                    .find(|&index| {
                        !fences.inside[index] && self.lines[index].trim_start().starts_with('#')
                    })
                    .unwrap_or(self.lines.len());
                last_content_line(&self.lines, next) + 1
            }
            None => last_content_line(&self.lines, self.lines.len()) + 1,
        }
    }

    /// Serialize back to text, preserving the original trailing-newline shape.
    pub fn to_text(&self) -> String {
        let mut out = self.lines.join(self.newline);
        if self.trailing_newline || !out.is_empty() {
            out.push_str(self.newline);
        }
        out
    }

    /// Write the file atomically.
    ///
    /// `fs::write` truncates first, so a write that fails partway (a full
    /// disk, a hit quota) leaves the person's criteria destroyed. Write a
    /// sibling temp file, flush it, then rename over the target, so a failure
    /// leaves the original byte-for-byte intact.
    pub fn save(&self) -> Result<()> {
        write_atomically(&self.path, &self.to_text())
            .with_context(|| format!("writing {}", self.path.display()))
    }

    /// The file's stem, used as its display name.
    pub fn name(&self) -> String {
        self.path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.display().to_string())
    }
}

/// Drop markdown emphasis around a token, so `**SEND-1**` reads as `SEND-1`.
pub fn strip_emphasis(token: &str) -> &str {
    token.trim_matches(|c| c == '*' || c == '_')
}

/// Drop a leading markdown bullet, if the line carries one.
fn strip_bullet(trimmed: &str) -> &str {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return rest.trim_start();
        }
    }
    trimmed
}

/// True when a trimmed line is shaped like a criterion: an optional bullet
/// followed by an id-shaped token.
fn is_criterion_line(trimmed: &str) -> bool {
    strip_bullet(trimmed)
        .split_whitespace()
        .next()
        .map(strip_emphasis)
        .is_some_and(looks_like_id)
}

/// Criterion-shaped lines in a file hi does not load as criteria.
///
/// `check` runs this over `Workspace::skipped` so an uppercase-named file in
/// `hi/` cannot swallow a criterion silently (hi: FILE-20). Returns the
/// zero-based line and the id-shaped token on it.
///
/// A fence makes its contents an example rather than structure, exactly as it
/// does under `## Intent` (hi: FILE-9). These files are prose all the way
/// down, so a documented example is never reported as a lost criterion.
pub fn criterion_tokens(text: &str) -> Vec<(usize, String)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut found = Vec::new();
    let mut fence: Option<(char, usize)> = None;

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(marker) = fence_marker(trimmed) {
            match fence {
                Some((open, len)) if open == marker.0 && marker.1 >= len => fence = None,
                Some(_) => {}
                None => fence = Some(marker),
            }
            continue;
        }
        if fence.is_some() || trimmed.is_empty() || !is_criterion_line(trimmed) {
            continue;
        }
        let token = strip_bullet(trimmed)
            .split_whitespace()
            .next()
            .map(strip_emphasis)
            .unwrap_or("")
            .to_string();
        found.push((index, token));
    }

    found
}

/// A ``` or ~~~ run at the start of a trimmed line, with its length.
///
/// Public because a fence means the same thing everywhere hi reads markdown:
/// what is inside it is an example rather than structure (hi: FILE-9).
/// `out::unwrap_soft_breaks` tracks fences for the same reason this does.
pub fn fence_marker(trimmed: &str) -> Option<(char, usize)> {
    for marker in ['`', '~'] {
        let len = trimmed.chars().take_while(|c| *c == marker).count();
        if len >= 3 {
            return Some((marker, len));
        }
    }
    None
}

/// Which body lines sit inside a fenced block, and where an unclosed fence
/// opens.
struct Fences {
    /// True for every line between a fence's markers, markers included.
    inside: Vec<bool>,
    /// The marker and the line of a fence that is never closed, when one is
    /// left open. The marker so a refusal can quote back what the person
    /// actually typed, tildes and all.
    unclosed: Option<(char, usize)>,
}

/// Run `fence_marker`'s open/close state machine over a whole file.
///
/// A fence is an example rather than structure everywhere hi reads markdown
/// (hi: FILE-9), which means everything that *writes* has to agree with the
/// parser about it too. Two verbs did not. `retired_heading` scanned raw lines
/// and found a `## Retired` inside somebody's fenced example, so `hi retire`
/// moved a real criterion into the intent prose and freed its id; and `insert`
/// appended a `## Criteria` section below an unfinished fence, which swallowed
/// it, so capture reported the same id saved twice and `hi check` saw nothing.
/// One state machine, one answer (hi: FILE-22, DECISIONS.md §31).
fn fence_map(lines: &[String], start: usize) -> Fences {
    let mut inside = vec![false; lines.len()];
    // Marker character, opening run length, and the line it opened on.
    let mut open: Option<(char, usize, usize)> = None;

    for (index, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.trim();
        match (fence_marker(trimmed), open) {
            (Some(marker), None) => {
                open = Some((marker.0, marker.1, index));
                inside[index] = true;
            }
            (Some(marker), Some((char, width, _))) => {
                inside[index] = true;
                if marker.0 == char && marker.1 >= width {
                    open = None;
                }
            }
            (None, state) => inside[index] = state.is_some(),
        }
    }

    Fences {
        inside,
        unclosed: open.map(|(marker, _, line)| (marker, line)),
    }
}

/// Walk back from `before` to the last line with content on it.
fn last_content_line(lines: &[String], before: usize) -> usize {
    let mut index = before.min(lines.len());
    while index > 0 && lines[index - 1].trim().is_empty() {
        index -= 1;
    }
    index.saturating_sub(1)
}

/// Collapse anything a person typed into something that is one line.
///
/// A criterion sentence has always gone through this. A retire reason did not,
/// so `hi retire SEND-1 "$(printf 'a\n- **SEND-5**  forged')"` wrote a second,
/// real criterion line into the file: `hi check` saw nothing wrong and SEND-5
/// was burned forever, an id nobody had written. Every string hi writes into a
/// file goes through here (hi: RETIRE-6).
pub fn one_line(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Render a criterion as one line: `ID  sentence`.
///
/// One criterion is always one line. Wrapping would make criteria harder to
/// grep, harder to diff, and harder to read as a list. The list is the point.
/// Internal whitespace is normalized so a pasted multi-line thought collapses
/// to a single sentence.
pub fn render_criterion(id: &Id, text: &str) -> Vec<String> {
    let sentence = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // A markdown list item, indented by depth. Bare lines would be joined into
    // one paragraph by every markdown renderer, so the file would read as a
    // wall of text everywhere it is actually looked at (hi: FILE-1, FILE-6).
    let indent = "  ".repeat(id.depth().saturating_sub(1));
    // The id is bold so it reads as a label rather than as the first two words
    // of the sentence, wherever the file is rendered.
    vec![format!("{indent}- **{id}**  {sentence}")]
}

/// The starting text for a brand-new feature file.
pub fn new_file_text(title: &str, family: &str) -> String {
    format!(
        "---\n\
         hi: 1\n\
         families: [{family}]\n\
         ---\n\
         \n\
         # {title}\n\
         \n\
         ## Intent\n\
         \n\
         <!-- What is this for, and what should it feel like? Write it as a person. -->\n\
         \n\
         ## Criteria\n\
         \n"
    )
}

/// Write `body` to `path` without ever leaving the target truncated.
pub fn write_atomically(path: &Path, body: &str) -> std::io::Result<()> {
    use std::io::Write;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    // A fixed temp name means two hi processes writing the same file share one
    // scratch path: the first rename moves it out from under the second, which
    // then fails with "no such file". Eight concurrent captures used to land
    // two. The pid makes the scratch file this process's own (hi: FILE-18).
    let temp = dir.join(format!(".{name}.{}.hi-tmp", std::process::id()));

    let attempt = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&temp)?;
        file.write_all(body.as_bytes())?;
        // Flush before the rename, so a failure is reported here rather than
        // silently producing a short file under the real name.
        file.flush()?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(err) = attempt {
        let _ = fs::remove_file(&temp);
        return Err(err);
    }

    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn an_empty_file_is_refused_rather_than_panicking() {
        // A zero-byte hi/*.md is what `touch`, a crashed editor or a partial
        // checkout leaves behind. It used to splice past the end of an empty
        // line buffer and abort with a backtrace (hi: CAPTURE-12).
        let mut doc = Doc::parse(PathBuf::from("/r/hi/empty.md"), "");
        let id = Id::parse("SEND-1").unwrap();
        assert!(
            doc.insert(&id, "a sentence").is_err(),
            "an empty file has no frontmatter, so the insert must refuse"
        );
    }
    use super::*;

    fn doc(raw: &str) -> Doc {
        Doc::parse(PathBuf::from("hi/chat.md"), raw)
    }

    const SAMPLE: &str = "---\nhi: 1\nfamilies: [SEND, RECEIPT]\nowner: leif\n---\n\n# Chat\n\n## Intent\n\nI want to talk to people I trust.\nIt should feel like texting.\n\n## Criteria\n\nSEND-1  I hit enter and the message shows up\n        right away, marked as sending.\nSEND-1.a  If I have no connection it queues.\nRECEIPT-1  The mark changes to sent.\n\n## Retired\n\nSEND-3  Messages auto-delete after 24 hours.\n        retired: different product\n";

    #[test]
    fn parses_frontmatter() {
        let doc = doc(SAMPLE);
        assert_eq!(doc.front.version, Some(1));
        assert_eq!(doc.front.families, vec!["SEND", "RECEIPT"]);
        assert_eq!(doc.front.owner.as_deref(), Some("leif"));
        assert_eq!(doc.title.as_deref(), Some("Chat"));
    }

    #[test]
    fn parses_intent_prose() {
        let doc = doc(SAMPLE);
        assert_eq!(
            doc.intent,
            "I want to talk to people I trust.\nIt should feel like texting."
        );
    }

    #[test]
    fn joins_continuation_lines() {
        let doc = doc(SAMPLE);
        assert_eq!(doc.criteria.len(), 3);
        assert_eq!(
            doc.criteria[0].text,
            "I hit enter and the message shows up right away, marked as sending."
        );
        assert_eq!(doc.criteria[0].line_no(), 16);
    }

    #[test]
    fn separates_retired_criteria() {
        let doc = doc(SAMPLE);
        assert_eq!(doc.retired.len(), 1);
        assert_eq!(doc.retired[0].raw_id, "SEND-3");
        assert_eq!(doc.retired[0].note.as_deref(), Some("different product"));
        // The reason is a note, not part of the sentence.
        assert_eq!(doc.retired[0].text, "Messages auto-delete after 24 hours.");
    }

    #[test]
    fn ignores_prose_inside_the_criteria_section() {
        let doc = doc("## Criteria\n\nJust a note to self.\n\nSEND-1  A real one.\n");
        assert_eq!(doc.criteria.len(), 1);
        assert_eq!(doc.criteria[0].raw_id, "SEND-1");
    }

    #[test]
    fn records_malformed_ids_rather_than_skipping_them() {
        let doc = doc("## Criteria\n\nSEND-1.a.b  Two letters in a row.\n");
        assert_eq!(doc.criteria.len(), 1);
        assert!(doc.criteria[0].id.is_none());
        assert!(doc.criteria[0].id_error.is_some());
    }

    #[test]
    fn inserts_beneath_the_last_descendant_of_a_parent() {
        let mut doc = doc(SAMPLE);
        let id = Id::parse("SEND-1.b").unwrap();
        doc.insert(&id, "If the thread was deleted it warns me.")
            .unwrap();
        let text = doc.to_text();
        let send_1a = text.find("SEND-1.a").unwrap();
        let send_1b = text.find("SEND-1.b").unwrap();
        let receipt = text.find("RECEIPT-1").unwrap();
        assert!(send_1a < send_1b, "new case goes after the existing one");
        assert!(send_1b < receipt, "and stays inside its own family block");
    }

    #[test]
    fn inserts_after_the_last_criterion_of_the_family() {
        let mut doc = doc(SAMPLE);
        let id = Id::parse("SEND-2").unwrap();
        doc.insert(&id, "It reaches them.").unwrap();
        let text = doc.to_text();
        assert!(text.find("SEND-2").unwrap() < text.find("RECEIPT-1").unwrap());
    }

    #[test]
    fn declares_a_new_family_in_frontmatter_on_insert() {
        let mut doc = doc(SAMPLE);
        let id = Id::parse("OFFLINE-1").unwrap();
        doc.insert(&id, "I can read old threads offline.").unwrap();
        assert!(doc.to_text().contains("families: [SEND, RECEIPT, OFFLINE]"));
    }

    #[test]
    fn insert_keeps_the_rest_of_the_file_byte_identical() {
        let mut doc = doc(SAMPLE);
        doc.insert(&Id::parse("SEND-2").unwrap(), "It reaches them.")
            .unwrap();
        let text = doc.to_text();
        assert!(text.contains("## Retired"));
        assert!(text.contains("        retired: different product"));
        assert!(text.contains("I want to talk to people I trust."));
    }

    #[test]
    fn round_trips_a_file_it_does_not_change() {
        let doc = doc(SAMPLE);
        assert_eq!(doc.to_text(), SAMPLE);
    }

    #[test]
    fn a_criterion_is_always_exactly_one_line() {
        let id = Id::parse("SEND-1").unwrap();
        let long = "I hit enter and the message shows up right away in the thread, marked as sending, so I never have to wonder whether it went.";
        let lines = render_criterion(&id, long);
        assert_eq!(lines.len(), 1, "no wrapping, however long the sentence");
        assert_eq!(lines[0], format!("- **SEND-1**  {long}"));
    }

    #[test]
    fn a_case_is_rendered_as_a_nested_list_item() {
        // Markdown joins consecutive plain lines into one paragraph, so every
        // criterion has to be a list item or the file reads as a wall of text.
        assert_eq!(
            render_criterion(&Id::parse("SEND-1").unwrap(), "One."),
            vec!["- **SEND-1**  One."]
        );
        assert_eq!(
            render_criterion(&Id::parse("SEND-1.a").unwrap(), "A case."),
            vec!["  - **SEND-1.a**  A case."]
        );
        assert_eq!(
            render_criterion(&Id::parse("SEND-1.a.1").unwrap(), "A step."),
            vec!["    - **SEND-1.a.1**  A step."]
        );
    }

    #[test]
    fn reads_a_criterion_however_it_was_decorated() {
        let doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- SEND-1  A plain bullet.\nSEND-2  No bullet at all.\n  - **SEND-2.a**  Bulleted and bold.\n  - _SEND-2.b_  Italic, because someone will.\n",
        );
        assert_eq!(doc.criteria.len(), 4);
        // The id is stored as it means, never as it was decorated.
        assert_eq!(doc.criteria[0].raw_id, "SEND-1");
        assert_eq!(doc.criteria[0].text, "A plain bullet.");
        assert_eq!(doc.criteria[1].raw_id, "SEND-2");
        assert_eq!(doc.criteria[2].raw_id, "SEND-2.a");
        assert_eq!(doc.criteria[2].text, "Bulleted and bold.");
        assert_eq!(doc.criteria[3].raw_id, "SEND-2.b");
    }

    #[test]
    fn collapses_pasted_whitespace_into_one_sentence() {
        let id = Id::parse("SEND-1").unwrap();
        let lines = render_criterion(&id, "I hit enter\n   and it   shows up");
        assert_eq!(lines, vec!["- **SEND-1**  I hit enter and it shows up"]);
    }

    #[test]
    fn appends_into_an_empty_criteria_section() {
        let mut doc = doc("---\nhi: 1\nfamilies: []\n---\n\n# Billing\n\n## Criteria\n\n");
        doc.insert(&Id::parse("BILLING-1").unwrap(), "I can see what I paid.")
            .unwrap();
        let text = doc.to_text();
        assert!(text.contains("**BILLING-1**  I can see what I paid."));
        assert!(text.contains("families: [BILLING]"));
    }

    #[test]
    fn keeps_the_blank_line_under_an_empty_criteria_heading() {
        let mut doc = doc("---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Criteria\n\n");
        doc.insert(&Id::parse("SEND-1").unwrap(), "I hit enter.")
            .unwrap();
        assert!(
            doc.to_text()
                .contains("## Criteria\n\n- **SEND-1**  I hit enter."),
            "heading keeps its blank line:\n{}",
            doc.to_text()
        );
    }

    #[test]
    fn opens_a_new_block_for_a_new_family() {
        let mut doc = doc("---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n");
        doc.insert(&Id::parse("RECEIPT-1").unwrap(), "Two.")
            .unwrap();
        assert!(
            doc.to_text()
                .contains("SEND-1  One.\n\n- **RECEIPT-1**  Two."),
            "families are separated by a blank line:\n{}",
            doc.to_text()
        );
    }

    #[test]
    fn keeps_one_family_in_one_block() {
        let mut doc = doc("---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n");
        doc.insert(&Id::parse("SEND-2").unwrap(), "Two.").unwrap();
        assert!(
            doc.to_text().contains("SEND-1  One.\n- **SEND-2**  Two."),
            "no blank line inside a family block:\n{}",
            doc.to_text()
        );
    }

    #[test]
    fn a_level_one_heading_closes_the_criteria_section() {
        // Regression: the `# ` branch used to clear the section without recording
        // the append point, so a new family landed above the existing block.
        let mut doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n\n# Appendix\n\nNotes.\n",
        );
        doc.insert(&Id::parse("RECEIPT-1").unwrap(), "Two.")
            .unwrap();
        let text = doc.to_text();
        assert!(
            text.find("SEND-1").unwrap() < text.find("RECEIPT-1").unwrap(),
            "a new family appends below the existing block:\n{text}"
        );
        assert!(
            text.find("RECEIPT-1").unwrap() < text.find("# Appendix").unwrap(),
            "and stays inside the criteria section:\n{text}"
        );
    }

    #[test]
    fn records_a_criterion_stranded_outside_every_section() {
        let doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  Seen.\n\n# Appendix\n\nSEND-9  Invisible.\n",
        );
        assert_eq!(doc.criteria.len(), 1);
        assert_eq!(doc.stray.len(), 1);
        assert_eq!(doc.stray[0].1, "SEND-9");
    }

    #[test]
    fn reads_block_style_families() {
        let doc = doc(
            "---\nhi: 1\nfamilies:\n  - SEND\n  - RECEIPT\nowner: leif\n---\n\n## Criteria\n\nSEND-1  One.\n",
        );
        assert_eq!(doc.front.families, vec!["SEND", "RECEIPT"]);
        assert!(doc.front.families_block);
        assert_eq!(doc.front.owner.as_deref(), Some("leif"));
        assert_eq!(doc.criteria.len(), 1);
    }

    #[test]
    fn keeps_block_style_when_adding_a_family() {
        let mut doc = doc("---\nhi: 1\nfamilies:\n  - SEND\n---\n\n## Criteria\n\nSEND-1  One.\n");
        doc.insert(&Id::parse("RECEIPT-1").unwrap(), "Two.")
            .unwrap();
        let text = doc.to_text();
        assert!(
            text.contains("families:\n  - SEND\n  - RECEIPT\n"),
            "the file's own style survives:\n{text}"
        );
        assert!(text.contains("**RECEIPT-1**  Two."));
        // And the criterion still landed in the right place.
        assert!(text.find("SEND-1").unwrap() < text.find("RECEIPT-1").unwrap());
    }

    #[test]
    fn block_style_insert_keeps_line_positions_correct() {
        // The frontmatter grows by one line, so everything below it shifts.
        let mut doc = doc("---\nhi: 1\nfamilies:\n  - SEND\n---\n\n## Criteria\n\nSEND-1  One.\n");
        doc.insert(&Id::parse("RECEIPT-1").unwrap(), "Two.")
            .unwrap();
        let text = doc.to_text();
        let reparsed = Doc::parse(PathBuf::from("hi/chat.md"), &text);
        for criterion in &doc.criteria {
            let line = &reparsed.lines[criterion.line];
            assert!(
                line.starts_with(&criterion.raw_id),
                "{} thinks it is on line {} but that line is {line:?}",
                criterion.raw_id,
                criterion.line
            );
        }
    }

    #[test]
    fn a_fenced_block_in_intent_is_not_parsed_as_criteria() {
        // Documenting the format inside your own intent must not create criteria.
        let doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Intent\n\nA file looks like:\n\n```markdown\n## Criteria\n\nSEND-9  an example, not a real one\n```\n\n## Criteria\n\nSEND-1  A real one.\n",
        );
        assert_eq!(doc.criteria.len(), 1, "only the real criterion counts");
        assert_eq!(doc.criteria[0].raw_id, "SEND-1");
        assert!(doc.stray.is_empty(), "the example is not a stray either");
        assert!(
            doc.intent.contains("```markdown"),
            "the fence stays in the prose"
        );
    }

    #[test]
    fn a_tilde_fence_is_honored_too() {
        let doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n~~~\nSEND-9  inside a fence\n~~~\nSEND-1  A real one.\n",
        );
        assert_eq!(doc.criteria.len(), 1);
        assert_eq!(doc.criteria[0].raw_id, "SEND-1");
    }

    #[test]
    fn crlf_line_endings_survive_a_write() {
        let raw =
            "---\r\nhi: 1\r\nfamilies: [SEND]\r\n---\r\n\r\n## Criteria\r\n\r\nSEND-1  One.\r\n";
        let mut doc = doc(raw);
        doc.insert(&Id::parse("SEND-2").unwrap(), "Two.").unwrap();
        let text = doc.to_text();
        assert!(text.contains("**SEND-2**  Two."));
        // Every newline is a CRLF: hi did not convert the file's endings.
        assert_eq!(
            text.matches('\n').count(),
            text.matches("\r\n").count(),
            "no bare LF may appear:\n{text:?}"
        );
        assert!(text.contains("**SEND-2**  Two.\r\n"));
    }

    #[test]
    fn a_normal_save_leaves_no_temp_file_behind() {
        let dir = std::env::temp_dir().join("hi-doc-save");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chat.md");
        fs::write(
            &path,
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n",
        )
        .unwrap();

        let mut doc = Doc::load(&path).unwrap();
        doc.insert(&Id::parse("SEND-1").unwrap(), "One.").unwrap();
        doc.save().unwrap();

        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("**SEND-1**  One.")
        );
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("hi-tmp"))
            .collect();
        assert!(leftovers.is_empty(), "the temp file must be renamed away");
    }

    #[test]
    fn retiring_takes_the_cases_with_it() {
        let mut doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n  - **SEND-1.a**  A case.\n- **SEND-2**  Two.\n",
        );
        let taken = doc
            .retire(&Id::parse("SEND-1").unwrap(), Some("different product"))
            .unwrap();
        assert_eq!(
            taken,
            vec!["SEND-1.a"],
            "the case goes with its parent, and is named"
        );
        let text = doc.to_text();
        assert!(text.contains("## Retired"));
        assert!(text.contains("retired: different product"));
        // SEND-2 stays live, and nothing is orphaned.
        let reparsed = Doc::parse(PathBuf::from("hi/chat.md"), &text);
        assert_eq!(reparsed.criteria.len(), 1);
        assert_eq!(reparsed.criteria[0].raw_id, "SEND-2");
        assert_eq!(reparsed.retired.len(), 2);
    }

    #[test]
    fn the_reason_attaches_to_the_criterion_it_explains() {
        // Regression: written after the whole block, the note parsed as a
        // continuation of the last case, so hi produced files that failed its
        // own check.
        let mut doc = doc(
            "---\nhi: 1\nfamilies: [GIFT]\n---\n\n## Criteria\n\n- **GIFT-1**  As an operator, I can cancel a gift.\n  - **GIFT-1.a**  As a member, I am told.\n",
        );
        doc.retire(&Id::parse("GIFT-1").unwrap(), Some("never shipped"))
            .unwrap();
        let reparsed = Doc::parse(PathBuf::from("hi/gift.md"), &doc.to_text());

        let parent = reparsed
            .retired
            .iter()
            .find(|c| c.raw_id == "GIFT-1")
            .unwrap();
        let case = reparsed
            .retired
            .iter()
            .find(|c| c.raw_id == "GIFT-1.a")
            .unwrap();
        assert_eq!(
            parent.note.as_deref(),
            Some("never shipped"),
            "the parent owns the reason"
        );
        assert_eq!(case.note, None, "the case does not");
    }

    #[test]
    fn a_reason_can_be_added_after_the_fact() {
        let mut doc = doc(
            "---\nhi: 1\nfamilies: [GIFT]\n---\n\n## Criteria\n\n- **GIFT-4**  As an operator, I can cancel a gift.\n",
        );
        let id = Id::parse("GIFT-4").unwrap();
        doc.retire(&id, None).unwrap();
        assert!(!doc.to_text().contains("retired:"));

        doc.set_retired_reason(&id, "we never shipped gifting")
            .unwrap();
        assert!(doc.to_text().contains("retired: we never shipped gifting"));

        // And saying it twice replaces rather than stacks.
        doc.set_retired_reason(&id, "different product").unwrap();
        let text = doc.to_text();
        assert_eq!(text.matches("retired:").count(), 1, "{text}");
        assert!(text.contains("retired: different product"));
    }

    #[test]
    fn retiring_without_a_reason_is_allowed() {
        let mut doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n\n## Retired\n\n- **SEND-9**  Older.\n",
        );
        doc.retire(&Id::parse("SEND-1").unwrap(), None).unwrap();
        let text = doc.to_text();
        assert!(!text.contains("retired:"), "no reason, no note");
        let reparsed = Doc::parse(PathBuf::from("hi/chat.md"), &text);
        assert_eq!(reparsed.retired.len(), 2);
        assert!(reparsed.criteria.is_empty());
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_frontmatter() {
        let doc = doc("\u{feff}---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n");
        assert_eq!(doc.front.version, Some(1));
        assert_eq!(doc.front.families, vec!["SEND"]);
        assert_eq!(doc.criteria.len(), 1);
    }

    #[test]
    fn a_file_with_no_criteria_heading_gets_one() {
        let mut doc =
            doc("---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nJust prose.\n");
        doc.insert(&Id::parse("SEND-1").unwrap(), "One.").unwrap();
        let text = doc.to_text();
        assert!(
            text.contains("## Criteria"),
            "a section is created:\n{text}"
        );
        assert!(
            text.find("## Criteria").unwrap() < text.find("SEND-1").unwrap(),
            "and the criterion lands under it:\n{text}"
        );
        // The prose is untouched and still parses as intent.
        let reparsed = Doc::parse(PathBuf::from("hi/chat.md"), &text);
        assert_eq!(reparsed.intent, "Just prose.");
        assert_eq!(reparsed.criteria.len(), 1);
        assert!(reparsed.stray.is_empty());
    }

    #[test]
    fn a_hash_comment_in_a_fenced_snippet_does_not_truncate_intent() {
        let doc = doc(
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Intent\n\nOne command:\n\n```bash\n# no config, no account\nbrew install hi\n```\n\nIt must stay that simple.\n\n## Criteria\n\nSEND-1  One.\n",
        );
        assert!(
            doc.intent.contains("It must stay that simple."),
            "prose after the fence survives: {:?}",
            doc.intent
        );
        assert_eq!(doc.criteria.len(), 1);
    }

    /// A file hi did not write: hand-typed, with a properly closed example of
    /// the format inside its own intent, exactly as `FILE-9` invites. Every
    /// write-path test used to assert on files hi itself had produced, and
    /// that is how these two got through (DECISIONS.md §26, §31).
    const DOCUMENTED: &str = "---\nhi: 1\nfamilies: [SEND]\n---\n\n\
         # Chat\n\n\
         ## Intent\n\n\
         Sending should feel instant. A file has three sections, and a criterion looks like this:\n\n\
         ```markdown\n\
         ## Intent\n\n\
         ## Criteria\n\n\
         - **SEND-4**  an example of the shape, not something anybody wants.\n\n\
         ## Retired\n\
         ```\n\n\
         That is all there is to it.\n\n\
         ## Criteria\n\n\
         - **SEND-1**  I hit enter and the message is on its way.\n";

    /// The fenced block of `DOCUMENTED`, which no verb may touch or read.
    const EXAMPLE: &str = "```markdown\n\
         ## Intent\n\n\
         ## Criteria\n\n\
         - **SEND-4**  an example of the shape, not something anybody wants.\n\n\
         ## Retired\n\
         ```";

    #[test]
    fn a_fenced_retired_example_is_not_the_retired_section() {
        // `retired_heading` scanned raw lines, so the `## Retired` inside the
        // example above was the section it retired into: the criterion landed
        // in the intent prose, `retire` printed success, and the file parsed
        // back with no criteria and no retirements at all. SEND-1 was then
        // free (hi: FILE-22, FILE-9, RETIRE-2).
        let mut file = doc(DOCUMENTED);
        let id = Id::parse("SEND-1").unwrap();
        file.retire(&id, Some("Dropped.")).unwrap();

        assert!(file.criteria.is_empty(), "it is no longer active");
        assert_eq!(file.retired.len(), 1, "and it is readable as retired");
        assert_eq!(file.retired[0].raw_id, "SEND-1");
        assert_eq!(file.retired[0].note.as_deref(), Some("Dropped."));

        // The example is prose and stays exactly as it was typed, and the
        // criterion inside it is still nobody's criterion.
        let after = file.to_text();
        assert!(
            after.contains(EXAMPLE),
            "the fenced example must be untouched:\n{after}"
        );
        // And the id is still spoken for, whichever way the file is read back.
        let reread = doc(&after);
        assert!(reread.all().any(|c| c.raw_id == "SEND-1"));
        assert!(
            !reread.all().any(|c| c.raw_id == "SEND-4"),
            "the example is an example (hi: FILE-9)"
        );
    }

    #[test]
    fn a_documented_example_is_left_completely_alone_by_a_capture() {
        // The other half of the same rule: the fix must not start treating a
        // fence as structure. The example keeps its lines and the new
        // criterion lands under the real heading below it (hi: FILE-9).
        let mut file = doc(DOCUMENTED);
        file.insert(&Id::parse("SEND-2").unwrap(), "It reaches them.")
            .unwrap();

        let text = file.to_text();
        let after = doc(&text);
        assert_eq!(
            after.criteria.len(),
            2,
            "the two real criteria, and only those: {text}"
        );
        assert_eq!(after.criteria[1].raw_id, "SEND-2");
        assert!(after.stray.is_empty(), "and nothing was stranded");
        assert!(
            text.contains(EXAMPLE),
            "the fenced example is byte-identical:\n{text}"
        );
        // SEND-4 is only ever drawn, so the id it draws is still free.
        assert!(
            file.insert(&Id::parse("SEND-4").unwrap(), "A real one.")
                .is_ok()
        );
    }

    #[test]
    fn a_capture_into_an_unfinished_fence_is_refused_rather_than_lost() {
        // `insert` appended a `## Criteria` section after the file's last line
        // with content, which here is inside a fence nobody closed, so the
        // heading and the criterion under it were both an example. Capture
        // reported success, `hi check` reported nothing, and the same id could
        // be "saved" over and over (hi: FILE-22, FILE-22.a, CAPTURE-5).
        let unfinished = "---\nhi: 1\nfamilies: [SEND]\n---\n\n\
             # Chat\n\n\
             ## Intent\n\n\
             Here is the shape I want, I was in the middle of writing it out:\n\n\
             ```markdown\n\
             ## Criteria\n";
        let mut file = doc(unfinished);
        let err = file
            .insert(&Id::parse("SEND-1").unwrap(), "I hit enter.")
            .unwrap_err()
            .to_string();

        assert!(err.contains("cannot read it back"), "{err}");
        assert!(err.contains("unclosed"), "and it says why: {err}");
        assert_eq!(
            file.to_text(),
            unfinished,
            "the unfinished prose is left exactly as it was"
        );

        // Closing the fence is all it takes, so the refusal is not a dead end.
        let mut file = doc(&format!("{unfinished}```\n"));
        file.insert(&Id::parse("SEND-1").unwrap(), "I hit enter.")
            .unwrap();
        assert_eq!(doc(&file.to_text()).criteria.len(), 1);
    }

    #[test]
    fn criterion_tokens_finds_a_criterion_in_prose_and_ignores_an_example() {
        let text = "# Notes\n\n\
                    - **SEND-9**  a criterion somebody put in the wrong file.\n\n\
                    ```\n\
                    - **SEND-4**  an example of the format.\n\
                    ```\n";
        let found = criterion_tokens(text);
        assert_eq!(found.len(), 1, "the fenced line is an example, not a loss");
        assert_eq!(found[0].1, "SEND-9");
        assert_eq!(found[0].0, 2);
    }

    #[test]
    fn new_file_text_parses_as_an_empty_doc() {
        let doc = doc(&new_file_text("Billing", "BILLING"));
        assert_eq!(doc.front.version, Some(1));
        assert_eq!(doc.front.families, vec!["BILLING"]);
        assert!(doc.criteria.is_empty());
    }
}
