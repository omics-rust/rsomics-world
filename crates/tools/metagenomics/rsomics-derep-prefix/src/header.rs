//! FASTA header manipulation: `;size=N` annotation parsing and serialisation.
//!
//! vsearch strips any existing `;size=N` field from the input label and
//! appends the newly computed abundance at the end, e.g.:
//!   Input:  `seq1;k=v;size=3;extra=x`
//!   Output: `seq1;k=v;extra=x;size=6`
//!
//! The field is identified as a semicolon-separated token that starts with
//! `size=` followed by digits.  Only the first such occurrence is stripped
//! (vsearch behaviour).

/// Parse and strip the first `;size=N` (or leading `size=N;`) from a label.
///
/// Returns `(stripped_label, Some(N))` if found, `(original, None)` otherwise.
#[must_use]
pub fn parse_size_annotation(label: &str) -> (String, Option<u64>) {
    // Split on ';', locate the first "size=<digits>" token.
    // Reconstruct by dropping that token only.
    let parts: Vec<&str> = label.split(';').collect();
    let mut size_value: Option<u64> = None;
    let mut kept: Vec<&str> = Vec::with_capacity(parts.len());

    for part in &parts {
        if size_value.is_none()
            && part.starts_with("size=")
            && let Ok(n) = part["size=".len()..].parse::<u64>()
        {
            size_value = Some(n);
            continue; // drop this token
        }
        kept.push(part);
    }

    let stripped = kept.join(";");
    (stripped, size_value)
}

/// Append `;size=N` to a label, writing to `out`.
pub fn write_header_with_size(
    out: &mut dyn std::io::Write,
    label: &str,
    abundance: u64,
) -> std::io::Result<()> {
    writeln!(out, ">{label};size={abundance}")
}

/// Strip the `;size=N` from a label (convenience wrapper over
/// [`parse_size_annotation`]).
#[must_use]
pub fn strip_size(label: &str) -> String {
    parse_size_annotation(label).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_middle() {
        let (s, v) = parse_size_annotation("seq1;k=v;size=3;extra=x");
        assert_eq!(s, "seq1;k=v;extra=x");
        assert_eq!(v, Some(3));
    }

    #[test]
    fn strip_trailing() {
        let (s, v) = parse_size_annotation("seq1;size=5");
        assert_eq!(s, "seq1");
        assert_eq!(v, Some(5));
    }

    #[test]
    fn no_size() {
        let (s, v) = parse_size_annotation("seq1;k=v");
        assert_eq!(s, "seq1;k=v");
        assert_eq!(v, None);
    }

    #[test]
    fn leading_size() {
        // Edge case: size is the first (and only) part after the name
        let (s, v) = parse_size_annotation("seq1;size=10");
        assert_eq!(s, "seq1");
        assert_eq!(v, Some(10));
    }

    #[test]
    fn non_numeric_size_kept() {
        let (s, v) = parse_size_annotation("seq1;size=abc");
        assert_eq!(s, "seq1;size=abc");
        assert_eq!(v, None);
    }
}
