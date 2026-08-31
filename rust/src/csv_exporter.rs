#[derive(Debug, Clone, Copy)]
pub struct PerfEntry {
    pub size: u64,
    pub throughput: f64,
}
struct CsvCol {
    name: &'static str,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CsvColRef(u16);

#[derive(Default)]
pub struct CsvData {
    cols: Vec<CsvCol>,
    rows_idx: Vec<CsvColRef>,
    rows: Vec<PerfEntry>,
}

impl CsvData {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(col_capacity: usize, rows_per_col: usize) -> Self {
        Self {
            cols: Vec::with_capacity(col_capacity),
            rows_idx: Vec::with_capacity(col_capacity * rows_per_col),
            rows: Vec::with_capacity(col_capacity * rows_per_col),
        }
    }

    pub fn col(&mut self, label: &'static str) -> CsvColRef {
        assert!(self.cols.len() <= u16::MAX as usize);
        let col_ref = CsvColRef(self.cols.len() as u16);
        self.cols.push(CsvCol { name: label });

        col_ref
    }

    pub fn row(&mut self, col_ref: CsvColRef, entry: PerfEntry) {
        assert!((col_ref.0 as usize) < self.cols.len());
        assert!(self.rows.len() < u16::MAX as usize);
        self.rows_idx.push(col_ref);
        self.rows.push(entry);
    }

    pub fn export(&self) -> Option<String> {
        if self.rows.len() == 0 || self.cols.len() == 0 {
            return None;
        }
        // Bytes,AName,BName,CName
        // n,A[n]Perf,B[n]Perf,C[n]Perf
        let sizes = {
            let mut sizes: Vec<u64> = Vec::with_capacity(self.rows.len());

            if sizes.len() < 32 {
                for item in &self.rows {
                    if sizes.contains(&item.size) {
                        continue;
                    } else {
                        sizes.push(item.size);
                    }
                }
            } else {
                for item in &self.rows {
                    sizes.push(item.size);
                }
                sizes.dedup();
            }
            sizes.sort();

            sizes
        };
        struct OrderedRef {
            entry_idx: u16,
            col: CsvColRef,
        }
        let mut row_refs: Vec<OrderedRef> = Vec::with_capacity(self.rows.len());
        for i in 0..self.rows.len() {
            row_refs.push(OrderedRef {
                entry_idx: i as u16,
                col: self.rows_idx[i],
            });
        }
        row_refs.sort_by(|a, b| {
            let data_ord =
                (self.rows[a.entry_idx as usize].size).cmp(&self.rows[b.entry_idx as usize].size);
            if data_ord.is_ne() {
                data_ord
            } else {
                a.col.0.cmp(&b.col.0)
            }
        });

        let mut buf: Vec<u8> = Vec::new(); // IDK actual size heurisitcs
        use std::io::Write as _;

        write!(buf, "Bytes").unwrap();

        for col in &self.cols {
            write!(buf, ",{}", col.name).unwrap();
        }
        write!(buf, "\n").unwrap();

        let mut refs_i: usize = 0;
        for i in 0..sizes.len() {
            let cur_size = sizes[i];
            write!(buf, "{}", cur_size).unwrap();
            let mut prev_col: Option<CsvColRef> = None;
            loop {
                let entry_ref = &row_refs[refs_i];
                let entry = self.rows[entry_ref.entry_idx as usize];
                let size = entry.size;

                let delta = entry_ref.col.0 as i32 - prev_col.map(|it| it.0 as i32).unwrap_or(-1);
                assert!(
                    delta > 0,
                    "multiple records for {} {}",
                    self.cols[entry_ref.col.0 as usize].name,
                    size
                );

                write!(
                    buf,
                    "{},{}",
                    ",".repeat((delta - 1) as usize),
                    entry.throughput
                )
                .unwrap();

                prev_col = Some(entry_ref.col);
                refs_i += 1;

                if refs_i >= row_refs.len()
                    || self.rows[row_refs[refs_i].entry_idx as usize].size != cur_size
                {
                    break;
                }
            }
            write!(
                buf,
                "{}\n",
                ",".repeat(self.cols.len() - 1 - prev_col.unwrap_or(CsvColRef(0)).0 as usize)
            )
            .unwrap();
        }
        return unsafe { Some(String::from_utf8_unchecked(buf)) };
    }
}

#[cfg(test)]
mod tests {
    use crate::csv_exporter::{CsvData, PerfEntry};

    #[test]
    fn exporter_1() {
        let mut csv_exporter = CsvData::new();

        let col1 = csv_exporter.col("A");
        let col2 = csv_exporter.col("B");

        csv_exporter.row(
            col1,
            PerfEntry {
                size: 32,
                throughput: 1.0,
            },
        );

        csv_exporter.row(
            col2,
            PerfEntry {
                size: 32,
                throughput: 1.0,
            },
        );

        csv_exporter.row(
            col2,
            PerfEntry {
                size: 64,
                throughput: 1.0,
            },
        );

        let res = csv_exporter.export().unwrap();
        insta::assert_snapshot!(res);
    }

    #[test]
    fn exporter_col_add_ordering() {
        let mut csv_exporter = CsvData::new();

        let col1 = csv_exporter.col("A");
        let col2 = csv_exporter.col("B");

        csv_exporter.row(
            col1,
            PerfEntry {
                size: 32,
                throughput: 1.0,
            },
        );

        csv_exporter.row(
            col2,
            PerfEntry {
                size: 32,
                throughput: 1.0,
            },
        );

        let col3 = csv_exporter.col("C");

        csv_exporter.row(
            col2,
            PerfEntry {
                size: 64,
                throughput: 1.0,
            },
        );

        csv_exporter.row(
            col3,
            PerfEntry {
                size: 128,
                throughput: 228.0,
            },
        );

        let res = csv_exporter.export().unwrap();
        insta::assert_snapshot!(res);
    }
}
