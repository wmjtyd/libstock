//! The standard identifier of a file in `libstock`.

use concat_string::concat_string;

/// Get the identifier according to the filename and timestamp.
///
/// The format of timestamp is `%Y%m%d`.
/// See [`super::timestamp::fmt_timestamp`] for more information.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::file::ident::get_ident;
///
/// let ident = get_ident("test", "20190101");
/// assert_eq!(ident, "test20190101");
/// ```
#[deprecated(since = "0.4.0", note = "We don't use this identifier anymore.")]
pub fn get_ident(filename: &str, timestamp: &str) -> String {
    concat_string!(filename, timestamp)
}
