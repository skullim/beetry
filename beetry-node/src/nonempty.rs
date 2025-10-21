use beetry_core::BoxNode;
use mitsein::vec1::Vec1;

pub type NonEmptyNodes = Vec1<BoxNode>;

pub trait Indices {
    fn indices(&self) -> std::ops::Range<usize>;
}

impl Indices for NonEmptyNodes {
    fn indices(&self) -> std::ops::Range<usize> {
        0..self.len().into()
    }
}
