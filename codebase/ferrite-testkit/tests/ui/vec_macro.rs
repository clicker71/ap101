//! Negative case: `vec!` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let items = vec![1u32, 2, 3];
        items.len()
    });
    let _ = value;
}
