use proptest::prelude::*;
use rsomics_intervals::{Interval, IntervalSet, coverage_bases, intersect, merge, subtract};

fn arb_interval() -> impl Strategy<Value = Interval> {
    (0u64..1000, 1u64..500).prop_map(|(start, len)| Interval {
        chrom: "chr1".to_string(),
        start,
        end: start + len,
        strand: None,
    })
}

fn arb_interval_set(max_len: usize) -> impl Strategy<Value = IntervalSet> {
    prop::collection::vec(arb_interval(), 0..max_len).prop_map(|ivs| {
        let mut set = IntervalSet::new();
        for iv in ivs {
            set.push(iv);
        }
        set
    })
}

fn coverage(set: &IntervalSet) -> u64 {
    coverage_bases(set).iter().map(|(_, bp)| bp).sum()
}

proptest! {
    #[test]
    fn merge_is_idempotent(set in arb_interval_set(20)) {
        let once = merge(&set);
        let twice = merge(&once);
        prop_assert_eq!(coverage(&once), coverage(&twice));
        prop_assert_eq!(once.len(), twice.len());
    }

    #[test]
    fn merge_does_not_increase_coverage(set in arb_interval_set(20)) {
        let merged = merge(&set);
        let sum_lengths: u64 = set.iter().map(|iv| iv.end - iv.start).sum();
        prop_assert!(coverage(&merged) <= sum_lengths);
    }

    #[test]
    fn intersect_is_commutative_on_coverage(
        a in arb_interval_set(10),
        b in arb_interval_set(10),
    ) {
        let ab = intersect(&a, &b);
        let ba = intersect(&b, &a);
        prop_assert_eq!(coverage(&ab), coverage(&ba));
    }

    #[test]
    fn subtract_plus_intersect_eq_original(
        a in arb_interval_set(10),
        b in arb_interval_set(10),
    ) {
        let a_merged = merge(&a);
        let a_sub_b = subtract(&a_merged, &b);
        let a_and_b = intersect(&a_merged, &b);
        let lhs = coverage(&a_sub_b) + coverage(&a_and_b);
        let rhs = coverage(&a_merged);
        prop_assert_eq!(lhs, rhs, "subtract + intersect should equal original coverage");
    }

    #[test]
    fn merge_result_has_no_overlaps(set in arb_interval_set(20)) {
        let merged = merge(&set);
        let ivs: Vec<&Interval> = merged.iter().collect();
        for w in ivs.windows(2) {
            if w[0].chrom == w[1].chrom {
                prop_assert!(
                    w[0].end <= w[1].start,
                    "overlap in merged: {:?} and {:?}", w[0], w[1]
                );
            }
        }
    }

    #[test]
    fn intersect_subset_of_both(
        a in arb_interval_set(10),
        b in arb_interval_set(10),
    ) {
        let ab = intersect(&a, &b);
        let ca = coverage(&merge(&a));
        let cb = coverage(&merge(&b));
        let cab = coverage(&merge(&ab));
        prop_assert!(cab <= ca, "intersect coverage {} > a coverage {}", cab, ca);
        prop_assert!(cab <= cb, "intersect coverage {} > b coverage {}", cab, cb);
    }
}
