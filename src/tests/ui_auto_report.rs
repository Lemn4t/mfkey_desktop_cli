use super::*;

#[test]
fn dict_merge_detail_reports_existing_count() {
    assert_eq!(
        render_dict_merge_detail(Some(4), 2),
        "existing dict has 4 key(s); adding 2 new"
    );
}

#[test]
fn dict_merge_detail_reports_no_existing_dict() {
    assert_eq!(
        render_dict_merge_detail(None, 5),
        "no existing dict on device, creating a new one"
    );
}
