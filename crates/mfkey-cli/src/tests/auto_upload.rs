use super::*;

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn parse_existing_keys_trims_uppercases_and_drops_blanks() {
    let data = b"aabbccddeeff\n  112233445566  \n\nAABBCCDDEEFF\n";
    let parsed = parse_existing_keys(data);
    assert_eq!(parsed, set(&["AABBCCDDEEFF", "112233445566"]));
}

#[test]
fn parse_existing_keys_on_empty_input_is_empty() {
    assert!(parse_existing_keys(b"").is_empty());
    assert!(parse_existing_keys(b"   \n\n  \n").is_empty());
}

#[test]
fn merge_key_sets_reports_only_new_keys() {
    let existing = set(&["AABBCCDDEEFF", "112233445566"]);
    let new = set(&["112233445566", "998877665544"]);
    let (added, merged) = merge_key_sets(&existing, &new);
    assert_eq!(added, 1);
    assert_eq!(
        merged,
        set(&["AABBCCDDEEFF", "112233445566", "998877665544"])
    );
}

#[test]
fn merge_key_sets_with_empty_existing_adds_all() {
    let new = set(&["A", "B"]);
    let (added, merged) = merge_key_sets(&BTreeSet::new(), &new);
    assert_eq!(added, 2);
    assert_eq!(merged, new);
}

#[test]
fn merge_key_sets_reports_zero_when_new_is_subset() {
    let existing = set(&["A", "B", "C"]);
    let new = set(&["A", "B"]);
    let (added, merged) = merge_key_sets(&existing, &new);
    assert_eq!(added, 0);
    assert_eq!(merged, existing);
}
