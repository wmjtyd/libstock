//! The standard identifier of a file in `libstock`.

use concat_string::concat_string;

/// Get the identifier according to the filename and timestamp.
///
/// &The format of timestamp is `%Y%m%d`.
/// See [`super::timestamp::fmt_timestamp`] for more information.
pub fn get_ident(filename: &str, timestamp: &str) -> String {
    concat_string!(filename, timestamp)
}
