//! Negative case: `Box::new` inside a 0-alloc section.

fn main() {
    let value = ferrite_macros::zero_alloc!({
        let boxed = Box::new(1u32);
        *boxed
    });
    let _ = value;
}
