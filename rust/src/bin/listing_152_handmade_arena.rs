use std::ptr::null_mut;

use haversine_generator::{arena::TypedArena, rep_run, setup_rep_test};

struct LinkedListNode {
    value: u32,
    next: *mut LinkedListNode,
}

struct LinkedListNodeBox {
    value: u32,
    next: Option<Box<LinkedListNodeBox>>,
}

fn main() {
    let mut rep_tester = setup_rep_test().unwrap();

    for i in 8..=20 {
        let count = 2usize.pow(i);
        let name = format!("arena {}", count);
        rep_run!(
            rep_tester,
            name = &name,
            len = count * size_of::<LinkedListNode>(),
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

        let name = format!("glibc {}", count);
        rep_run!(
            rep_tester,
            name = &name,
            len = count * size_of::<LinkedListNodeBox>(),
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
    }
}
