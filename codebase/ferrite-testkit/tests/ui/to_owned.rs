//! Negative case: `.to_owned()` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let text = "x".to_owned();
        text.len()
    });
    let _ = value;
}
