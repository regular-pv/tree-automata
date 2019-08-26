use std::hash::Hash;
use std::fmt;

#[cfg(not(debug_assertions))]
pub trait Label = Hash + Clone + Eq;

#[cfg(debug_assertions)]
pub trait Label = Hash + Clone + Eq + fmt::Display + fmt::Debug;

// impl Label for () {}
// impl Label for String {}
// impl<'a> Label for &'a str {}
// impl Label for char {}
// impl Label for bool {}
// impl Label for u8 {}
// impl Label for u16 {}
// impl Label for u32 {}
// impl Label for u64 {}
// impl Label for i8 {}
// impl Label for i16 {}
// impl Label for i32 {}
// impl Label for i64 {}

pub type Labeled<T, L> = (T, L);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct NoLabel;

impl fmt::Display for NoLabel {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

// impl<T, L: Label> Labeled<T, L> {
//     pub fn new(t: T, label: L) -> Labeled<T, L> {
//         Labeled {
//             t: t,
//             label: label
//         }
//     }
// }
//
// impl<T: Clone, L: Label> Clone for Labeled<T, L> {
//     fn clone(&self) -> Self {
//         Labeled {
//             t: self.t.clone(),
//             label: self.label.clone()
//         }
//     }
// }
//
// impl<T: Hash, L: Label> Hash for Labeled<T, L> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.t.hash(state);
//         self.label.hash(state);
//     }
// }
//
// impl<T: PartialEq, L: Label> PartialEq for Labeled<T, L> {
//     fn eq(&self, other: &Labeled<T, L>) -> bool {
//         self.label == other.label && self.t == other.t
//     }
// }
//
// impl<T: Eq, L: Label> Eq for Labeled<T, L> { }
