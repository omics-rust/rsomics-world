//! FASTA header manipulation: `;size=N` annotation parsing.

/// Parse and strip the first `;size=N` token from a label.
///
/// Returns `(stripped_label, Some(N))` if found, `(original, None)` otherwise.
/// Only the first `size=<digits>` semicolon-separated token is stripped
/// (vsearch behaviour).
#[must_use]
pub fn parse_size_annotation(label: &str) -> (String, Option<u64>) {
    let parts: Vec<&str> = label.split(';').collect();
    let mut size_value: Option<u64> = None;
    let mut kept: Vec<&str> = Vec::with_capacity(parts.len());

    for part in &parts {
        if size_value.is_none()
            && part.starts_with("size=")
            && let Ok(n) = part["size=".len()..].parse::<u64>()
        {
            size_value = Some(n);
            continue;
        }
        kept.push(part);
    }

    let stripped = kept.join(";");
    (stripped, size_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_trailing() {
        let (s, v) = parse_size_annotation("seq1;size=5");
        assert_eq!(s, "seq1");
        assert_eq!(v, Some(5));
    }

    #[test]
    fn strip_middle() {
        let (s, v) = parse_size_annotation("seq1;k=v;size=3;extra=x");
        assert_eq!(s, "seq1;k=v;extra=x");
        assert_eq!(v, Some(3));
    }

    #[test]
    fn no_size() {
        let (s, v) = parse_size_annotation("seq1;k=v");
        assert_eq!(s, "seq1;k=v");
        assert_eq!(v, None);
    }

    #[test]
    fn non_numeric_kept() {
        let (s, v) = parse_size_annotation("seq1;size=abc");
        assert_eq!(s, "seq1;size=abc");
        assert_eq!(v, None);
    }
}
