#[macro_export]
macro_rules! with_label {
    ($label:path => $($body:tt)+) => {
        let __mark = if (cfg!(feature="profiler")) {
            Some($crate::simple_profiler::core::mark_scope($label as u32))
        } else {
            None
        };

        $($body)+

        #[allow(unreachable_code)]
        drop(__mark);
    };
}
#[macro_export]
macro_rules! with_label_expr {
    ($label:path => $body:block) => {{
        let __mark = if (cfg!(feature="profiler")) {
            Some($crate::simple_profiler::core::mark_scope($label as u32))
        } else {
            None
        };

        let __result = $body;

        drop(__mark);

        __result
    }};
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
        if (cfg!(feature="profiler")) {
            $crate::simple_profiler::core::start_profile();
        }

        $($t)+

        if (cfg!(feature="profiler")) {
            $crate::simple_profiler::core::finish_end_print_root_profile($labels::ALL).unwrap();
        }
    };
}

profiling_labels! {
    enum Labels {
        Outer = 1,
        Inner
    }
}

pub fn main() {
    with_profiling! { Labels =>
        with_label! {
            Labels::Outer =>

            let a = 10;
            let b = a + 10;
        };
    }

    println!("{}", b)
}
