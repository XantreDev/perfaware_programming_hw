use std::{env, ptr::null_mut, u32};

use haversine_generator::{
    arena::TypedArena,
    csv_exporter::{self, CsvData},
    csv_exporter_utils::CsvImport,
    rep_run,
    rep_tester::MeasurementKind,
    setup_rep_test,
};

struct LinkedListNode {
    value: u32,
    next: *mut LinkedListNode,
}

struct LinkedListNodeBox {
    value: u32,
    next: Option<Box<LinkedListNodeBox>>,
}

struct LinkedListDataOriented<T> {
    nodes: Vec<T>,
    refs: Vec<IndexOrU32>,
}
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct IndexOrU32(u32);

impl IndexOrU32 {
    const NONE: IndexOrU32 = IndexOrU32(u32::MAX);

    pub fn get(&self) -> Option<u32> {
        if self.0 == Self::NONE.0 {
            return None;
        } else {
            return Some(self.0);
        }
    }
}

struct LinkedListDataOrientedEntry<T> {
    value: T,
    next: IndexOrU32,
}
impl<T> LinkedListDataOriented<T> {
    fn with_capacity(capacity: usize) -> Self {
        LinkedListDataOriented {
            nodes: Vec::with_capacity(capacity),
            refs: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, item: LinkedListDataOrientedEntry<T>) {
        self.nodes.push(item.value);
        self.refs.push(item.next);
    }
}

fn main() {
    let mut rep_tester = setup_rep_test().unwrap();
    // let use_csv = {
    //     let env_var = env::var_os("CSV").unwrap_or("0".into());

    //     env_var == "1" || env_var == "true"
    // };
    let mut csv_exporter = CsvData::with_capacity(3, 11);
    let arena_col = csv_exporter.col("arena");
    let glibc_col = csv_exporter.col("glibc");
    let data_oriented_col = csv_exporter.col("data-oriented");

    for i in 10..=20 {
        let count = 2usize.pow(i);
        let len = count * size_of::<LinkedListNode>();
        let name = format!("arena {} ({}kB)", count, len / 1024);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            before = {
                let arena = TypedArena::new();
            },
            block = {
                let mut start = LinkedListNode {
                    value: 0,
                    next: null_mut(),
                };
                let mut head = &mut start;
                for j in 0..(count as u32) {
                    let new_head = arena.alloc_unwrap(LinkedListNode {
                        value: j * 10,
                        next: null_mut(),
                    });
                    head.next = new_head;
                    head = new_head;
                }
            },
        );
        csv_exporter.measurement_row(arena_col, rep_tester.measurement(MeasurementKind::Best));

        let len = count * size_of::<LinkedListNodeBox>();
        let name = format!("glibc {} ({}kB)", count, len / 1024);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            before = {},
            block = {
                let mut start = LinkedListNodeBox {
                    value: 0,
                    next: None,
                };
                let mut head = unsafe { &mut start as *mut LinkedListNodeBox };
                for j in 0..(count as u32) {
                    let next = Box::new(LinkedListNodeBox {
                        value: j * 10,
                        next: None,
                    });

                    unsafe {
                        (*head).next = Some(next);
                        head = &mut start as *mut LinkedListNodeBox
                    }
                }
            },
            // check = { item.value == (count as u64 - 1) * 10 }
        );
        csv_exporter.measurement_row(glibc_col, rep_tester.measurement(MeasurementKind::Best));

        let len = count * (size_of::<u32>() + size_of::<IndexOrU32>()) + 48;
        let name = format!("data_oriented {} ({}kB)", count, len / 1024);
        rep_run!(
            rep_tester,
            name = &name,
            len = len,
            block = {
                let mut list: LinkedListDataOriented<u32> =
                    LinkedListDataOriented::with_capacity(count);

                for j in 0..(count as u32) {
                    list.push(LinkedListDataOrientedEntry {
                        value: j * 10,
                        next: IndexOrU32(j + 1),
                    });
                }

                if count != 0 {
                    list.refs[count as usize - 1] = IndexOrU32::NONE;
                }
            }
        );
        csv_exporter.measurement_row(
            data_oriented_col,
            rep_tester.measurement(MeasurementKind::Best),
        );
    }
    csv_exporter.export().inspect(|v| {
        println!("\n{}", v);
    });
}
