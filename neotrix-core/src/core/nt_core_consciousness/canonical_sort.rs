/// Canonical sort for deterministic ordering of consciousness subsystems.
pub fn sort_e8_hexagrams<T, F>(items: &mut [T], key: F)
where
    F: Fn(&T) -> u64,
{
    items.sort_by_key(|item| key(item));
}

pub fn sort_gwt_specialists<'a, T: 'a, F>(items: &mut [T], name: F)
where
    F: Fn(&T) -> &'a str,
{
    items.sort_by(|a, b| name(a).cmp(name(b)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_e8_hexagrams() {
        let mut items = vec![(3u64, "c"), (1u64, "a"), (2u64, "b")];
        sort_e8_hexagrams(&mut items, |x| x.0);
        assert_eq!(items[0].1, "a");
        assert_eq!(items[1].1, "b");
        assert_eq!(items[2].1, "c");
    }

    #[test]
    fn test_sort_gwt_specialists() {
        let mut items = vec!["z", "m", "a"];
        sort_gwt_specialists(&mut items, |x| *x);
        assert_eq!(items, vec!["a", "m", "z"]);
    }

    #[test]
    fn test_empty_slice() {
        let mut items: Vec<u64> = vec![];
        sort_e8_hexagrams(&mut items, |x| *x);
        assert!(items.is_empty());
    }

    #[test]
    fn test_single_element() {
        let mut items = vec![42u64];
        sort_e8_hexagrams(&mut items, |x| *x);
        assert_eq!(items, vec![42]);
    }
}
