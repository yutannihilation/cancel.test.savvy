// Example functions

use savvy::savvy;

/// Convert Input To Upper-Case
///
/// @param x A character vector.
/// @returns A character vector with upper case version of the input.
/// @export
#[savvy]
fn fun() -> savvy::Result<()> {
    for i in 0..30 {
        savvy::r_println!("Iteration {i}");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
