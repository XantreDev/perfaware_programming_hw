#[derive(Debug, Clone, Copy)]
struct Entry {
    size: u64,
    throughput: f64,
}

#[derive(Debug, Clone)]
struct CsvCol {
    label: &'static str,
    points: Vec<Entry>,
}

struct CsvData {
    cols: Vec<CsvCol>,
}

impl CsvData {
    fn new() -> Self {
        CsvData {
            cols: Vec::with_capacity(8),
        }
    }

    fn create_col(&mut self, label: &'static str) {
        self.cols.push(CsvCol {
            label,
            // bunch of allocations :(
            points: Vec::with_capacity(32),
        });
    }
}
