use beetry_macros::assert_unique_types;

struct T;
struct R;
struct S;

assert_unique_types!(T);
assert_unique_types!(T, R, S);

fn main() {}
