/// Debug macro, lifted from the std
#[macro_export]
macro_rules! dbg {
    // No argument case: prints the file and line number
    () => {
        $crate::println!("[{}:{}]", file!(), line!());
    };

    // Single expression: prints the value of the expression
    ($val:expr) => {
        // Using `match` to ensure temporary lifetimes are handled correctly
        match &$val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), tmp);
                tmp
            }
        }
    };

    // A case where a trailing comma is allowed but not necessary
    ($val:expr,) => {
        $crate::dbg!($val)
    };

    // Multiple expressions: prints each expression with a trailing comma safely handled
    ($($val:expr),+ $(,)?) => {
        // Use an iterator to avoid building a tuple and provide a more optimized expansion
        let _ = ($($crate::dbg!($val)),+,);  // Silently discard the tuple as it’s for debug output only
    };
}