//! Negative case: `String::new` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let text = String::new();
        text.len()
    });
    let _ = value;
}
