#![feature(trait_alias)]

extern crate terms;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialOrd, Ord, Ordering};
pub use terms::Ranked;

mod state;
mod label;
mod language;
mod inter;
mod utils;
pub mod bottom_up;
pub mod alternating;

pub use state::*;
pub use label::*;
pub use language::*;
pub use inter::*;

pub use utils::*;

#[cfg(not(debug_assertions))]
pub trait Symbol = Hash + Clone + Eq;

#[cfg(debug_assertions)]
pub trait Symbol = Hash + Clone + Eq + fmt::Display + fmt::Debug;

pub struct Rank<T>(pub T, pub usize);

impl<T> Ranked for Rank<T> {
    fn arity(&self) -> usize {
        self.1
    }
}

impl<T: Ord> Ord for Rank<T> {
    fn cmp(&self, other: &Rank<T>) -> Ordering {
        match self.1.cmp(&other.1) {
            Ordering::Equal => self.0.cmp(&other.0),
            ord => ord
        }
    }
}

impl<T: PartialOrd> PartialOrd for Rank<T> {
    fn partial_cmp(&self, other: &Rank<T>) -> Option<Ordering> {
        match self.1.cmp(&other.1) {
            Ordering::Equal => self.0.partial_cmp(&other.0),
            ord => Some(ord)
        }
    }
}

impl<T: Eq> Eq for Rank<T> {}

impl<T: PartialEq> PartialEq for Rank<T> {
    fn eq(&self, other: &Rank<T>) -> bool {
        self.1 == other.1 && self.0 == other.0
    }
}

impl<T: Hash> Hash for Rank<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<T: Copy> Copy for Rank<T> {}

impl<T: Clone> Clone for Rank<T> {
    fn clone(&self) -> Rank<T> {
        Rank(self.0.clone(), self.1)
    }
}

impl<T: fmt::Display> fmt::Display for Rank<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl<T: fmt::Debug> fmt::Debug for Rank<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}:{}", self.0, self.1)
    }
}

// Sorted instances.
pub trait SortedWith<T> {
    fn sort(&self) -> &T;
}

impl<A, B> SortedWith<B> for (A, B) {
    fn sort(&self) -> &B {
        &self.1
    }
}

pub struct Sorted<X, T>(pub X, pub T);

impl<X, T> Sorted<X, T> {
    pub fn new(x: X, t: T) -> Sorted<X, T> {
        Sorted(x, t)
    }
}

impl<X, T> SortedWith<T> for Sorted<X, T> {
    fn sort(&self) -> &T {
        &self.1
    }
}

impl<X: Ord, T: Ord> Ord for Sorted<X, T> {
    fn cmp(&self, other: &Sorted<X, T>) -> Ordering {
        match self.1.cmp(&other.1) {
            Ordering::Equal => self.0.cmp(&other.0),
            ord => ord
        }
    }
}

impl<X: PartialOrd, T: PartialOrd> PartialOrd for Sorted<X, T> {
    fn partial_cmp(&self, other: &Sorted<X, T>) -> Option<Ordering> {
        match self.1.partial_cmp(&other.1) {
            Some(Ordering::Equal) => self.0.partial_cmp(&other.0),
            Some(ord) => Some(ord),
            None => None
        }
    }
}

impl<X: Eq, T: Eq> Eq for Sorted<X, T> {}

impl<X:PartialEq, T: PartialEq> PartialEq for Sorted<X, T> {
    fn eq(&self, other: &Sorted<X, T>) -> bool {
        self.1 == other.1 && self.0 == other.0
    }
}

impl<X: Hash, T: Hash> Hash for Sorted<X, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<X: Copy, T: Copy> Copy for Sorted<X, T> {}

impl<X: Clone, T: Clone> Clone for Sorted<X, T> {
    fn clone(&self) -> Sorted<X, T> {
        Sorted(self.0.clone(), self.1.clone())
    }
}

impl<X: fmt::Display, T: fmt::Display> fmt::Display for Sorted<X, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl<X: fmt::Debug, T: fmt::Debug> fmt::Debug for Sorted<X, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}:{:?}", self.0, self.1)
    }
}
