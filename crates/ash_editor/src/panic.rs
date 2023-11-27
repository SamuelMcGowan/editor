use std::cell::RefCell;
use std::panic::{catch_unwind, set_hook, take_hook, UnwindSafe};

use backtrace::Backtrace;
use color_backtrace::termcolor::{ColorChoice, StandardStream};
use color_backtrace::BacktracePrinter;

thread_local! {
    static SAVED_PANIC: RefCell<Option<SavedPanic>> = RefCell::default();
}

struct SavedPanic {
    message: String,
    trace: Backtrace,
}

#[must_use]
pub fn catch_and_reprint_panic<T>(f: impl FnOnce() -> T + UnwindSafe) -> Option<T> {
    let prev_hook = take_hook();

    set_hook(Box::new(|panic_info| {
        let saved_panic = SavedPanic {
            message: format!("{panic_info}"),
            trace: Backtrace::new(),
        };
        SAVED_PANIC.replace(Some(saved_panic));
    }));

    let result = catch_unwind(f);

    set_hook(prev_hook);

    match result {
        Ok(result) => Some(result),

        Err(_) => {
            let saved_panic = SAVED_PANIC
                .take()
                .expect("Panicked but could not retrieve saved panic.");

            BacktracePrinter::new()
                .message(saved_panic.message)
                .print_trace(
                    &saved_panic.trace,
                    &mut StandardStream::stderr(ColorChoice::Auto),
                )
                .expect("Failed to print backtrace.");

            None
        }
    }
}
