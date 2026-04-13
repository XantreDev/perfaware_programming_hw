#[macro_export]
macro_rules! unix_extern {
    (
        $(fn $name:ident($($arg:ident:$type:ty),*) $( -> $ret:ty)?;)*
    ) => {
        #[cfg(unix)]
        unsafe extern "C" {
            $(
                pub unsafe fn $name($($arg:$type),*) $( -> $ret:ty)?;
            )*
        }

        $(
            #[cfg(not(unix))]
            pub unsafe extern "C" fn $name($($arg:$type),*) $( -> $ret:ty)? {
                panic!("TODO")
            }
        )*
    };
}
