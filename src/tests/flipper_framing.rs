use super::*;

fn feed(bytes: &[u8]) -> impl FnMut() -> Result<u8> + '_ {
    let mut it = bytes.iter().copied();
    move || it.next().ok_or(FlipperError::Timeout)
}

#[test]
fn encode_varint_matches_known_values() {
    assert_eq!(encode_varint(0), vec![0x00]);
    assert_eq!(encode_varint(1), vec![0x01]);
    assert_eq!(encode_varint(127), vec![0x7f]);
    assert_eq!(encode_varint(128), vec![0x80, 0x01]);
    assert_eq!(encode_varint(300), vec![0xac, 0x02]);
    assert_eq!(encode_varint(1_000_000), vec![0xc0, 0x84, 0x3d]);
}

#[test]
fn varint_round_trips() {
    for n in [
        0u64,
        1,
        127,
        128,
        255,
        300,
        16384,
        1_000_000,
        u32::MAX as u64,
    ] {
        let bytes = encode_varint(n);
        let decoded = decode_varint(feed(&bytes)).unwrap();
        assert_eq!(decoded, n);
    }
}

#[test]
fn decode_varint_rejects_too_long() {
    let bytes = [0x80u8; 10];
    assert!(decode_varint(feed(&bytes)).is_err());
}

#[test]
fn validate_frame_len_rejects_zero() {
    assert!(validate_frame_len(0).is_err());
}

#[test]
fn validate_frame_len_rejects_too_large() {
    assert!(validate_frame_len(MAX_FRAME_LEN + 1).is_err());
}

#[test]
fn validate_frame_len_accepts_normal_sizes() {
    assert_eq!(validate_frame_len(42).unwrap(), 42);
    assert_eq!(validate_frame_len(MAX_FRAME_LEN).unwrap(), MAX_FRAME_LEN);
}
