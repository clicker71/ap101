//! Negative case: `Vec::new` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let items = Vec::new();
        items.len()
    });
    let _ = value;
}
