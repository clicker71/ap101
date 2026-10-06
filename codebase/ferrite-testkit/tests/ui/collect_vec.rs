//! Negative case: `.collect::<Vec<_>>()` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let items = [1u32, 2, 3].iter().collect::<Vec<_>>();
        items.len()
    });
    let _ = value;
}
