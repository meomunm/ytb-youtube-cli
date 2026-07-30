//! Parsing 1-based result indices from the command line.
//!
//! Core invariant: **validate every index before executing anything**. A
//! command like `play 1 99 3` must fail up front (99 is out of range) without
//! playing 1 first — otherwise the user gets a half-executed action.

use crate::err::AppError;

/// Parse a single argument into a 1-based index.
///
/// Accepts only a non-empty run of ASCII digits denoting a value `>= 1`. This
/// rejects `"1a"`, `"-1"`, `" 1"`, `"1.0"`, and `"0"`.
pub fn parse_one(s: &str) -> Option<usize> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<usize>().ok().filter(|&n| n >= 1)
}

/// Parse and validate all index arguments against a list of length `len`.
///
/// Returns the indices in the exact order given (duplicates preserved), or the
/// first error encountered. No index is returned unless *all* are valid.
pub fn parse_all(args: &[String], len: usize) -> Result<Vec<usize>, AppError> {
    let mut out = Vec::with_capacity(args.len());
    for a in args {
        match parse_one(a) {
            None => return Err(AppError::BadIndex(a.clone())),
            Some(n) if n > len => return Err(AppError::IndexOutOfRange { got: n, max: len }),
            Some(n) => out.push(n),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_one_accepts_positive_digits() {
        assert_eq!(parse_one("1"), Some(1));
        assert_eq!(parse_one("42"), Some(42));
    }

    #[test]
    fn parse_one_rejects_zero_and_junk() {
        assert_eq!(parse_one("0"), None);
        assert_eq!(parse_one(""), None);
        assert_eq!(parse_one("1a"), None);
        assert_eq!(parse_one("-1"), None);
        assert_eq!(parse_one(" 1"), None);
        assert_eq!(parse_one("1.0"), None);
    }

    #[test]
    fn parse_all_preserves_order_and_dups() {
        assert_eq!(
            parse_all(&v(&["3", "1", "1", "2"]), 3).unwrap(),
            vec![3, 1, 1, 2]
        );
    }

    #[test]
    fn parse_all_rejects_bad_index() {
        assert_eq!(
            parse_all(&v(&["abc"]), 5),
            Err(AppError::BadIndex("abc".to_string()))
        );
    }

    #[test]
    fn parse_all_rejects_out_of_range() {
        assert_eq!(
            parse_all(&v(&["6"]), 5),
            Err(AppError::IndexOutOfRange { got: 6, max: 5 })
        );
    }

    #[test]
    fn parse_all_validates_everything_before_returning() {
        // "1" is fine but "99" is not: the whole call must fail.
        assert_eq!(
            parse_all(&v(&["1", "99", "3"]), 10),
            Err(AppError::IndexOutOfRange { got: 99, max: 10 })
        );
    }

    #[test]
    fn parse_all_empty_is_empty() {
        assert_eq!(parse_all(&[], 5).unwrap(), Vec::<usize>::new());
    }
}
