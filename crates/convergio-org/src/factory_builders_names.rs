//! Human name generator for AI agents.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const FIRST: &[&str] = &[
    "marco",
    "sara",
    "luca",
    "elena",
    "andrea",
    "giulia",
    "alex",
    "chiara",
    "matteo",
    "sofia",
    "giovanni",
    "laura",
    "davide",
    "valentina",
    "paolo",
    "martina",
    "simone",
    "alessia",
    "fabio",
    "silvia",
    "enrico",
    "monica",
    "carlo",
    "diana",
    "omar",
    "nadia",
    "tommy",
    "irene",
    "leo",
    "aurora",
];
const LAST: &[&str] = &[
    "rossi",
    "bianchi",
    "ferrari",
    "romano",
    "colombo",
    "ricci",
    "marino",
    "greco",
    "bruno",
    "gallo",
    "costa",
    "giordano",
    "mancini",
    "rizzo",
    "lombardi",
    "moretti",
    "barbieri",
    "fontana",
    "santoro",
    "mariani",
    "vitale",
    "serra",
    "conti",
    "fabbri",
    "gentile",
    "caruso",
    "leone",
    "pellegrini",
    "testa",
    "parisi",
];

/// Pick a deterministic human name from the pool based on org slug + role hint.
pub fn human_name(slug: &str, role_hint: &str) -> String {
    let mut h = DefaultHasher::new();
    slug.hash(&mut h);
    role_hint.hash(&mut h);
    let hash = h.finish();
    let first = FIRST[(hash as usize) % FIRST.len()];
    let last = LAST[((hash >> 16) as usize) % LAST.len()];
    format!("{first}-{last}-ai")
}
