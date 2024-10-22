use savvy::savvy;
use std::{
    cell::LazyCell,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    },
};

unsafe extern "C" {
    fn Rf_onintr();
}

#[derive(Debug)]
struct Foo {}

impl Drop for Foo {
    fn drop(&mut self) {
        savvy::r_eprintln!("dropped!");
    }
}

static INTERRUPTED: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| {
    let interrupted = Arc::new(AtomicBool::new(false));
    let i = interrupted.clone();

    ctrlc::set_handler(move || {
        i.store(true, Ordering::SeqCst);
    })
    .unwrap();

    interrupted
});

/// @export
#[savvy]
fn fun() -> savvy::Result<()> {
    // To check if objects are properly dropped
    let foo = Foo {};

    INTERRUPTED.store(false, Ordering::SeqCst);

    for i in 0..30 {
        if INTERRUPTED.load(Ordering::SeqCst) {
            savvy::r_println!("Interrupted");
            unsafe {
                savvy::unwind_protect(|| {
                    Rf_onintr();
                    savvy::NullSexp.into()
                })?;
            }
            return Ok(());
        }
        savvy::r_println!("Iteration {i}");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    savvy::r_println!("{foo:?}");

    Ok(())
}
