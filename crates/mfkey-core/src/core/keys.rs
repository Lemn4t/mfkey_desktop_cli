use std::collections::BTreeSet;

pub fn parse_key_lines(data: &[u8]) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for line in String::from_utf8_lossy(data).lines() {
        let l = line.trim();
        if !l.is_empty() {
            set.insert(l.to_uppercase());
        }
    }
    set
}

pub fn merge_key_sets(
    existing: &BTreeSet<String>,
    new: &BTreeSet<String>,
) -> (usize, BTreeSet<String>) {
    let mut merged = existing.clone();
    let before = merged.len();
    for k in new {
        merged.insert(k.clone());
    }
    let added = merged.len() - before;
    (added, merged)
}

#[cfg(test)]
#[path = "../tests/keys.rs"]
mod tests;
