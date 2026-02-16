#[macro_export]
macro_rules! mark_scope {
    (@mark $label: path, $bytes: expr) => {
        if (cfg!(feature = "profiler")) {
            Some($crate::simple_profiler::core::mark_scope(
                $label as u32,
                $bytes as u64,
            ))
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! with_label {
    (@inner $label:path, $bytes:expr, $($body:tt)+) => {
        let __mark = $crate::mark_scope!(@mark $label, $bytes);

        $($body)+

        #[allow(unreachable_code)]
        drop(__mark);
    };

    ($label:path => $($body:tt)+) => {
        with_label!(@inner $label, 0, $($body)+)
    };
    ($label:path where bytes=$bytes:expr => $($body:tt)+) => {
        with_label!(@inner $label, $bytes, $($body)+)
    };
}
#[macro_export]
macro_rules! with_label_fn {
    ($label:path => $vis:vis fn $name:ident($($arg:ident:$type:ty),* $(,)?) -> $ret:ty { $($body:tt)* }) => {
        $vis fn $name($($arg:$type),*) -> $ret {
            let __mark = $crate::mark_scope!(@mark $label, 0);

            $($body)*
        }
    };
}
#[macro_export]
macro_rules! with_label_expr {
    (@inner $label:path, $bytes: expr, $body:tt) => {{
        let __mark = $crate::mark_scope!(@mark $label, $bytes);

        let __result = $body;

        drop(__mark);

        __result
    }};

    ($label:path where bytes=$bytes:expr => $body:block) => {
        with_label_expr!(@inner $label, $bytes, $body)
    };
    ($label:path where bytes=$bytes:expr => $body:expr) => {
        with_label_expr!(@inner $label, $bytes, $body)
    };

    ($label:path => $body:block) => {
        with_label_expr!(@inner $label, 0, $body)
    };
    ($label:path => $body:expr) => {
        with_label_expr!($label => {$body})
    };
}

#[macro_export]
macro_rules! profiling_labels {
    ($vis:vis enum $name: ident {
        $first_ident:ident = $start:expr,
        $($i:ident),*
        $(,)?
    }) => {
        #[repr(u32)]
        $vis enum $name {
            $first_ident = $start,
            $($i),*
        }

        impl $name {
            $vis const ALL: &'static [(u32, &'static str)] = &[
                ($name::$first_ident as u32, stringify!($first_ident)),
                $(($name::$i as u32, stringify!($i))),*
            ];
            $vis const COUNT: usize = $name::ALL.len();
        }
    };
}

#[macro_export]
macro_rules! with_profiling {
    ($labels:ident => $($t: tt)+) => {
        $crate::simple_profiler::core::start_profile();

        $($t)+

        $crate::simple_profiler::core::finish_end_print_root_profile($labels::ALL).unwrap();
    };
}

// profiling_labels! {
//     enum Labels {
//         Outer = 1,
//         Inner
//     }
// }

// pub fn main() {
//     with_profiling! { Labels =>
//         with_label! {
//             Labels::Outer =>

//             let a = 10;
//             let b = a + 10;
//         };
//     }

//     println!("{}", b)
// }
