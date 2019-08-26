use std::hash::Hash;
#[cfg(debug_assertions)]
use std::fmt;

#[cfg(not(debug_assertions))]
pub trait State = Hash + Clone + Eq;

#[cfg(debug_assertions)]
pub trait State = Hash + Clone + Eq + fmt::Display + fmt::Debug;

// impl State for () {}
// impl State for String {}
// impl<'a> State for &'a str {}
// impl State for char {}
// impl State for bool {}
// impl State for u8 {}
// impl State for u16 {}
// impl State for u32 {}
// impl State for u64 {}
// impl State for i8 {}
// impl State for i16 {}
// impl State for i32 {}
// impl State for i64 {}
//
// impl<A: Hash + Clone + Eq, B: Hash + Clone + Eq> State for (A, B) {}
