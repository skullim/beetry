use beetry_macros::assert_unique_types;

struct T;
struct R;

assert_unique_types!(&'static T, &'static R);
assert_unique_types!(Option<T>, Result<T, R>);
assert_unique_types!((u32, i32), [u8; 4], fn(T) -> R);

fn main() {}
