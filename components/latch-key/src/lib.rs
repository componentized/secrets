//! `componentized:secrets` latch that decides each secret by its key, with patterns read from
//! `wasi:config/store`, see README.md.

use std::sync::OnceLock;

use exports::componentized::secrets::latch::{
    Decision, ErrorCode, GetArgs, Guest as LatchGuest, Operation, StoreErrorCode, StoreOperation,
};
use wasi::config::store as config;
use wasi::logging::logging::{Level, log};

const LATCH_NAME: &str = "latch-key";

struct Component;

impl LatchGuest for Component {
    fn authorize(operation: Operation) -> Result<Decision, ErrorCode> {
        let rules = RULES
            .get_or_init(load_rules)
            .as_ref()
            .map_err(Clone::clone)?;
        let Operation::Store(StoreOperation::Get(GetArgs { key })) = &operation;
        Ok(match decide(rules, key) {
            true => Decision::Denied(StoreErrorCode::NotFound),
            false => Decision::Deferred,
        })
    }

    fn observe_decision(_final_decision: Decision, _operation: Operation) -> Result<(), ErrorCode> {
        // no side effects
        Ok(())
    }
}

/// The rules, or the error loading them, loaded once by the first `authorize`.
static RULES: OnceLock<Result<Vec<Rule>, ErrorCode>> = OnceLock::new();

/// A pattern of keys, and whether the keys it matches are denied.
struct Rule {
    pattern: String,
    denied: bool,
}

/// Reads the rules from the config, each key a pattern and each value its decision. An invalid
/// config is logged and fails with `invalid-config`, for this and every later operation.
fn load_rules() -> Result<Vec<Rule>, ErrorCode> {
    let entries = config::get_all().map_err(|err| match err {
        config::Error::Upstream(message) => format!("ERROR=upstream: {message}"),
        config::Error::Io(message) => format!("ERROR=io: {message}"),
    });
    entries
        .and_then(|entries| {
            entries
                .into_iter()
                .map(|(pattern, value)| match value.as_str() {
                    "" | "deferred" => Ok(Rule {
                        pattern,
                        denied: false,
                    }),
                    "denied" => Ok(Rule {
                        pattern,
                        denied: true,
                    }),
                    _ => Err(format!(
                        "KEY={pattern} VALUE={value} ERROR=expected one of: 'deferred', 'denied'"
                    )),
                })
                .collect()
        })
        .map_err(|message| {
            log(
                Level::Critical,
                "componentized-latch",
                &format!("Invalid config LATCH={LATCH_NAME} {message}"),
            );
            ErrorCode::InvalidConfig(LATCH_NAME.to_string())
        })
}

/// Whether the key is denied. The matching rule with the most literal characters decides, a
/// denial when matching rules are equally specific. A key no rule matches is deferred.
fn decide(rules: &[Rule], key: &str) -> bool {
    rules
        .iter()
        .filter(|rule| matches(&rule.pattern, key))
        .max_by_key(|rule| (literals(&rule.pattern), rule.denied))
        .is_some_and(|rule| rule.denied)
}

/// The number of characters of the pattern that are not wildcards.
fn literals(pattern: &str) -> usize {
    pattern.chars().filter(|c| *c != '*').count()
}

/// Whether the pattern matches the whole key, `*` matching any characters, including none.
fn matches(pattern: &str, key: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let key: Vec<char> = key.chars().collect();
    let (mut p, mut k) = (0, 0);
    // the position after the last `*`, and the key position it is matching from
    let mut star: Option<(usize, usize)> = None;
    while k < key.len() {
        if p < pattern.len() && pattern[p] == '*' {
            star = Some((p + 1, k));
            p += 1;
        } else if p < pattern.len() && pattern[p] == key[k] {
            p += 1;
            k += 1;
        } else if let Some((after_star, from)) = star {
            // the last `*` matches one more character
            star = Some((after_star, from + 1));
            p = after_star;
            k = from + 1;
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|c| *c == '*')
}

wit_bindgen::generate!({
    path: "../wit",
    world: "latch",
    generate_all,
});

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_patterns() {
        assert!(matches("password", "password"));
        assert!(!matches("password", "passwords"));
        assert!(matches("*", ""));
        assert!(matches("*", "anything"));
        assert!(matches("db.*", "db.password"));
        assert!(!matches("db.*", "api.token"));
        assert!(matches("*.token", "api.token"));
        assert!(matches("a*b*c", "aXbYc"));
        assert!(matches("a*b*c", "abbc"));
        assert!(!matches("a*b*c", "aXbY"));
        assert!(matches("**", "x"));
    }

    fn rules(rules: &[(&str, bool)]) -> Vec<Rule> {
        rules
            .iter()
            .map(|(pattern, denied)| Rule {
                pattern: pattern.to_string(),
                denied: *denied,
            })
            .collect()
    }

    #[test]
    fn unmatched_key_is_deferred() {
        assert!(!decide(&rules(&[]), "password"));
        assert!(!decide(&rules(&[("db.*", true)]), "password"));
    }

    #[test]
    fn most_specific_rule_decides() {
        let rules = rules(&[("*", true), ("db.*", false), ("db.admin", true)]);
        assert!(decide(&rules, "api.token"));
        assert!(!decide(&rules, "db.password"));
        assert!(decide(&rules, "db.admin"));
    }

    #[test]
    fn denial_wins_a_tie() {
        assert!(decide(&rules(&[("a*", false), ("*a", true)]), "aa"));
        assert!(decide(&rules(&[("*a", true), ("a*", false)]), "aa"));
    }
}
