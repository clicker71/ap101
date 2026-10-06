//! Negative case: `Vec::with_capacity` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let items = Vec::with_capacity(4);
        items.len()
    });
    let _ = value;
}
