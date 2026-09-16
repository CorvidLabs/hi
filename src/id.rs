//! Criterion identifiers.
//!
//! An id is a hand-written family, a hyphen, and a dotted path of levels:
//!
//! ```text
//! SEND-1        a criterion              (number)
//! SEND-1.a        a case of it           (letter)
//! SEND-1.a.1        a step in that case  (number)
//! SEND-1.a.1.b        a case of that     (letter)
//! ```
//!
//! Levels alternate strictly: number, letter, number, letter. `SEND-1.a.b` is
//! rejected, because a case of a case has to be expressed as a step containing
//! cases.
//! Ids are permanent and append-first; nothing here ever renumbers anything.

use std::fmt;

/// One level of an id's dotted path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// A step or detail. Odd depths (the first level included).
    Number(u32),
    /// A case or branch. Even depths.
    Letter(String),
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Number(n) => write!(f, "{n}"),
            Level::Letter(s) => write!(f, "{s}"),
        }
    }
}

/// A parsed criterion id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id {
    pub family: String,
    pub levels: Vec<Level>,
}

/// Why an id string is not a valid id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    /// No `-` separating family from levels.
    MissingHyphen,
    /// Family is empty or not `[A-Z][A-Z0-9_]*`.
    BadFamily(String),
    /// No levels after the hyphen.
    NoLevels,
    /// A level was empty (a doubled or trailing dot).
    EmptyLevel,
    /// A level was neither a number nor lowercase letters.
    BadLevel(String),
    /// A numeric level carried a leading zero, so its spelling is ambiguous.
    PaddedLevel(String),
    /// A level was the right shape but the wrong kind for its depth.
    ///
    /// Carries the 1-based depth and what was expected there.
    Alternation {
        depth: usize,
        expected: &'static str,
    },
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdError::MissingHyphen => write!(f, "no '-' between family and number"),
            IdError::BadFamily(s) => {
                write!(
                    f,
                    "family '{s}' must start with A-Z and contain only A-Z, 0-9, _"
                )
            }
            IdError::NoLevels => write!(f, "nothing after the '-'"),
            IdError::EmptyLevel => write!(f, "empty level (a doubled or trailing '.')"),
            IdError::BadLevel(s) => {
                write!(f, "level '{s}' must be a number or lowercase letters")
            }
            IdError::PaddedLevel(s) => write!(
                f,
                "level '{s}' has a leading zero; write it as '{}', so the id always \
                 means the same thing",
                s.trim_start_matches('0')
            ),
            IdError::Alternation { depth, expected } => write!(
                f,
                "level {depth} must be {expected}, because levels alternate number, \
                 letter, number, letter"
            ),
        }
    }
}

impl Id {
    /// Parse an id, enforcing the family charset and strict level alternation.
    pub fn parse(raw: &str) -> Result<Id, IdError> {
        let (family, rest) = raw.split_once('-').ok_or(IdError::MissingHyphen)?;

        if family.is_empty() || !valid_family(family) {
            return Err(IdError::BadFamily(family.to_string()));
        }
        if rest.is_empty() {
            return Err(IdError::NoLevels);
        }

        let mut levels = Vec::new();
        for (index, part) in rest.split('.').enumerate() {
            if part.is_empty() {
                return Err(IdError::EmptyLevel);
            }
            // Depth 0 is a number, depth 1 a letter, and so on.
            let wants_number = index % 2 == 0;

            let level = if part.chars().all(|c| c.is_ascii_digit()) {
                // A padded level would parse to a different spelling than it was
                // written (`SEND-007` -> `SEND-7`), so every reference to it, an
                // exported parent or a captured case, would point somewhere that
                // does not exist. Refuse the ambiguity instead.
                if part.len() > 1 && part.starts_with('0') {
                    return Err(IdError::PaddedLevel(part.to_string()));
                }
                Level::Number(
                    part.parse::<u32>()
                        .map_err(|_| IdError::BadLevel(part.to_string()))?,
                )
            } else if part.chars().all(|c| c.is_ascii_lowercase()) {
                Level::Letter(part.to_string())
            } else {
                return Err(IdError::BadLevel(part.to_string()));
            };

            let is_number = matches!(level, Level::Number(_));
            if is_number != wants_number {
                return Err(IdError::Alternation {
                    depth: index + 1,
                    expected: if wants_number { "a number" } else { "a letter" },
                });
            }
            levels.push(level);
        }

        Ok(Id {
            family: family.to_string(),
            levels,
        })
    }

    /// The id one level up, or `None` for a top-level criterion.
    pub fn parent(&self) -> Option<Id> {
        if self.levels.len() <= 1 {
            return None;
        }
        let mut levels = self.levels.clone();
        levels.pop();
        Some(Id {
            family: self.family.clone(),
            levels,
        })
    }

    /// True when `self` sits anywhere beneath `other`.
    pub fn is_descendant_of(&self, other: &Id) -> bool {
        self.family == other.family
            && self.levels.len() > other.levels.len()
            && self.levels[..other.levels.len()] == other.levels[..]
    }

    /// How deep the id sits. A top-level criterion is depth 1.
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// The top-level number, used to find the next free id in a family.
    pub fn root_number(&self) -> Option<u32> {
        match self.levels.first() {
            Some(Level::Number(n)) => Some(*n),
            _ => None,
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-", self.family)?;
        for (index, level) in self.levels.iter().enumerate() {
            if index > 0 {
                write!(f, ".")?;
            }
            write!(f, "{level}")?;
        }
        Ok(())
    }
}

fn valid_family(family: &str) -> bool {
    let mut chars = family.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// True when a token is shaped like an id. Used to tell a criterion line from
/// prose without committing to the id being valid.
pub fn looks_like_id(token: &str) -> bool {
    let Some((family, rest)) = token.split_once('-') else {
        return false;
    };
    // Deliberately case-insensitive on the family, so a lowercase `send-2`
    // is recognized and then REJECTED by `parse` with a reason, rather than
    // silently read as prose and lost (hi: CHECK-2.d).
    let family_shaped = !family.is_empty()
        && family.starts_with(|c: char| c.is_ascii_alphabetic())
        && family
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    // The first level is always a number, which is what separates an id from
    // an ordinary hyphenated word like `spec-sync` or `well-formed`.
    let numbered = rest.starts_with(|c: char| c.is_ascii_digit());
    family_shaped && numbered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_top_level_id() {
        let id = Id::parse("SEND-1").unwrap();
        assert_eq!(id.family, "SEND");
        assert_eq!(id.levels, vec![Level::Number(1)]);
        assert_eq!(id.to_string(), "SEND-1");
        assert_eq!(id.depth(), 1);
        assert!(id.parent().is_none());
    }

    #[test]
    fn parses_alternating_levels() {
        let id = Id::parse("SEND-1.a.1.b").unwrap();
        assert_eq!(
            id.levels,
            vec![
                Level::Number(1),
                Level::Letter("a".into()),
                Level::Number(1),
                Level::Letter("b".into()),
            ]
        );
        assert_eq!(id.to_string(), "SEND-1.a.1.b");
    }

    #[test]
    fn rejects_two_letters_in_a_row() {
        let err = Id::parse("SEND-1.a.b").unwrap_err();
        assert_eq!(
            err,
            IdError::Alternation {
                depth: 3,
                expected: "a number"
            }
        );
    }

    #[test]
    fn rejects_a_letter_where_the_first_level_must_be_a_number() {
        let err = Id::parse("SEND-a").unwrap_err();
        assert_eq!(
            err,
            IdError::Alternation {
                depth: 1,
                expected: "a number"
            }
        );
    }

    #[test]
    fn rejects_bad_families() {
        assert!(matches!(Id::parse("send-1"), Err(IdError::BadFamily(_))));
        assert!(matches!(Id::parse("1SEND-1"), Err(IdError::BadFamily(_))));
        assert!(matches!(Id::parse("SE-ND-1"), Err(IdError::BadLevel(_))));
        assert!(matches!(Id::parse("SEND1"), Err(IdError::MissingHyphen)));
        assert!(matches!(Id::parse("SEND-"), Err(IdError::NoLevels)));
        assert!(matches!(Id::parse("SEND-1."), Err(IdError::EmptyLevel)));
        assert!(matches!(Id::parse("SEND-1..a"), Err(IdError::EmptyLevel)));
    }

    #[test]
    fn rejects_a_zero_padded_level() {
        // SEND-007 would parse to SEND-7, so every reference to it would dangle.
        assert!(matches!(
            Id::parse("SEND-007"),
            Err(IdError::PaddedLevel(_))
        ));
        assert!(matches!(
            Id::parse("SEND-1.a.01"),
            Err(IdError::PaddedLevel(_))
        ));
        // A bare zero is not padding, and an unpadded number is fine.
        assert!(Id::parse("SEND-0").is_ok());
        assert!(Id::parse("SEND-7").is_ok());
        assert!(Id::parse("SEND-10").is_ok());
    }

    #[test]
    fn allows_underscores_and_digits_in_a_family() {
        assert!(Id::parse("SEND_2FA-1").is_ok());
    }

    #[test]
    fn finds_parents() {
        let id = Id::parse("SEND-1.a.1").unwrap();
        assert_eq!(id.parent().unwrap().to_string(), "SEND-1.a");
        assert_eq!(id.parent().unwrap().parent().unwrap().to_string(), "SEND-1");
    }

    #[test]
    fn knows_its_descendants() {
        let root = Id::parse("SEND-1").unwrap();
        assert!(Id::parse("SEND-1.a").unwrap().is_descendant_of(&root));
        assert!(Id::parse("SEND-1.a.1").unwrap().is_descendant_of(&root));
        assert!(!Id::parse("SEND-2").unwrap().is_descendant_of(&root));
        assert!(!Id::parse("RECEIPT-1.a").unwrap().is_descendant_of(&root));
        // A criterion is not its own descendant.
        assert!(!root.is_descendant_of(&root));
        // SEND-11 must not read as a descendant of SEND-1.
        assert!(!Id::parse("SEND-11").unwrap().is_descendant_of(&root));
    }

    #[test]
    fn recognises_id_shaped_tokens() {
        assert!(looks_like_id("SEND-1"));
        assert!(looks_like_id("SEND-1.a"));
        // Wrong case is still plainly meant as an id, so it is recognized here
        // and rejected by `parse`, rather than vanishing into prose.
        assert!(looks_like_id("send-2"));
        assert!(looks_like_id("Send-3"));
        // An ordinary hyphenated word is not an id: the first level is a number.
        assert!(!looks_like_id("well-formed"));
        assert!(!looks_like_id("spec-sync"));
        assert!(!looks_like_id("co-authored"));
        assert!(!looks_like_id("SEND-a"));
        assert!(!looks_like_id("I"));
        assert!(!looks_like_id("Given"));
    }

    #[test]
    fn a_wrongly_cased_family_is_rejected_with_a_reason() {
        assert!(matches!(Id::parse("send-2"), Err(IdError::BadFamily(_))));
        assert!(matches!(Id::parse("Send-3"), Err(IdError::BadFamily(_))));
    }
}
