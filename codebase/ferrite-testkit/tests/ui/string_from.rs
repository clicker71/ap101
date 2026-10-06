//! Negative case: `String::from` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let text = String::from("x");
        text.len()
    });
    let _ = value;
}
