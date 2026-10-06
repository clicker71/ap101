//! Negative case: `format!` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let text = format!("{}", 42);
        text.len()
    });
    let _ = value;
}
