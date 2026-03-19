use beetry_macros::assert_unique_types;

assert_unique_types!(u32);
assert_unique_types!(u32, i32);
assert_unique_types!(u8, i16, u32, i64, usize, bool, char);

fn main() {}
