use savvy::{eval_parse_text, savvy};
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
        savvy::r_println!("dropped!");
    }
}

static INTERRUPTED: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

/// @export
#[savvy]
fn fun() -> savvy::Result<()> {
    // To check if objects are properly dropped
    let foo = Foo {};

    // Create a runtime.
    // cf. https://notgull.net/blocking-leaky/

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    std::thread::spawn({
        let handle = rt.handle().clone();

        // Run the handle on this thread, forever.
        move || handle.block_on(std::future::pending::<()>())
    });

    let _guard = rt.enter();

    rt.spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        INTERRUPTED.store(true, Ordering::SeqCst);
    });

    pollster::block_on(async {
        for i in 0..30 {
            savvy::r_println!("Iteration {i}");

            // This works, but why?
            // eval_parse_text("Sys.sleep(0)").unwrap();

            if INTERRUPTED.load(Ordering::SeqCst) {
                savvy::r_println!("Interrupted");
                unsafe {
                    savvy::unwind_protect(|| {
                        Rf_onintr();
                        savvy::NullSexp.into()
                    })
                    .unwrap();
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        savvy::r_println!("{foo:?}");
    });

    Ok(())
}
