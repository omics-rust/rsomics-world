import copy
import unittest

from compare_rnaseq_qc_workflow import METRICS, compare_report


class SharedReportComparisonTests(unittest.TestCase):
    def test_complete_report_rejects_missing_mapping_coverage_and_tin_population_changes(self):
        mapping = {"Total records:": 88, "Proper-paired reads map to different chrom:": 0}
        distribution = dict(totals={"Total Reads": 83, "Total Tags": 86, "Total Assigned Tags": 83},
                            groups={"CDS_Exons": dict(bases=2900, tags=51, tags_per_kb=17.58)})
        integrity = dict(transcripts=[dict(geneID="positive", chrom="chr1", tx_start=100,
                                          tx_end=200, TIN=100 / 102),
                                     dict(geneID="zero", chrom="chr1", tx_start=300,
                                          tx_end=400, TIN=0)],
                         summary=dict(mean=50 / 102, median=50 / 102, stdev=50 / 102),
                         eligible_transcripts=["positive", "zero"])
        report = dict(schema_version="1.0", profile="rseqc-5.0.5", sample="single", metrics=list(METRICS),
                      mapping=mapping, strandedness=dict(layout="SingleEnd", fractions={"same": 80 / 81}),
                      distribution=distribution, coverage=dict(sample="single", coverage=[1] * 100),
                      integrity=integrity)
        expected = {metric: copy.deepcopy(report[metric]) for metric in METRICS}
        expected["strandedness"]["fractions"]["same"] = .9877
        for row in expected["integrity"]["transcripts"]:
            row.update(tx_start=str(row["tx_start"]), tx_end=str(row["tx_end"]), TIN=str(row["TIN"]))
        self.assertEqual(compare_report(report, expected, "single")["eligible_transcripts"], 2)
        for change in ("mapping", "coverage", "tin_score", "tin_population"):
            altered = copy.deepcopy(report)
            if change == "mapping":
                del altered["mapping"]["Proper-paired reads map to different chrom:"]
            elif change == "coverage":
                altered["coverage"]["coverage"][49] += 1
            elif change == "tin_score":
                altered["integrity"]["transcripts"][0]["TIN"] = 0
            else:
                altered["integrity"]["eligible_transcripts"].pop()
            with self.subTest(change=change), self.assertRaises(ValueError):
                compare_report(altered, expected, "single")


if __name__ == "__main__":
    unittest.main()
