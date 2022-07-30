/// Convert a UNIX timestamp in `ms` to a 6-byte hex string.
///
/// Note that we will do the following check in debug mode:
///
/// - Make sure the encoded `u64` number do not use the 0 & 1 byte.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::num::unix_ms_to_six_byte_hex;
///
/// assert_eq!(unix_ms_to_six_byte_hex(1656991593000), [1, 129, 204, 101, 50, 40]);
/// ```
pub fn unix_ms_to_six_byte_hex(timestamp: u64) -> [u8; 6] {
    let encoded = timestamp.to_be_bytes();

    // Make sure the encoded number do not use the 0 & 1 byte.
    debug_assert_eq!(encoded[0], 0);
    debug_assert_eq!(encoded[1], 0);

    *arrayref::array_ref![encoded, 2, 6]
}

/// Convert 6-byte hex string to the UNIX timestamp in `ms`.
///
/// Note: we convert to `u64` instead of `u128`, as the latter
/// is not native and may introduce performance degradation.
/// Besides, `u128` is meaningless as the encoded_timestamp only support
/// numbers that can be represented with 6-byte hex string, as known as
/// the subset of `u64`.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::num::six_byte_hex_to_unix_ms;
///
/// assert_eq!(six_byte_hex_to_unix_ms(&[1, 129, 204, 101, 50, 40]), 1656991593000);
/// ```
pub fn six_byte_hex_to_unix_ms(encoded_timestamp: &[u8; 6]) -> u64 {
    let eight_byte_encoded = {
        let mut buf = [0u8; 8];
        arrayref::array_mut_ref![buf, 2, 6].copy_from_slice(encoded_timestamp);

        buf
    };

    u64::from_be_bytes(eight_byte_encoded)
}
