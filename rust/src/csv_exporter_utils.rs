use crate::{
    csv_exporter::{CsvColRef, CsvData, PerfEntry},
    rep_tester::PerformanceMeasurement,
};

pub trait CsvImport {
    fn measurement_row(&mut self, col_ref: CsvColRef, measurement: PerformanceMeasurement);
}

impl CsvImport for CsvData {
    fn measurement_row(&mut self, col_ref: CsvColRef, measurement: PerformanceMeasurement) {
        self.row(
            col_ref,
            PerfEntry {
                size: measurement.bytes,
                throughput: measurement.throughput_mb(),
            },
        )
    }
}
