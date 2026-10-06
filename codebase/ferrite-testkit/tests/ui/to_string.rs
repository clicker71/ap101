//! Negative case: `.to_string()` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let text = "x".to_string();
        text.len()
    });
    let _ = value;
}
