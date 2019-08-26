use crate::{Symbol, State};
use crate::bottom_up::Configuration;

pub trait Language<F> {
    // ...
}

pub trait ConfigurationIterator<'a, F: Symbol, Q: State> : Iterator<Item = Configuration<F, Q>> {
    // ...
}
