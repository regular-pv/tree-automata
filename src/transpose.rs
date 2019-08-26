trait Transpose<P> {
    /// Return the transposed version of the language.
    fn transpose(&self, pattern: P) -> Self;
}

impl<F, Q, L, P> Transpose<P> for Automaton<F, Q, L> {
    fn transpose(&self, pattern: P) -> Automaton<F, Q, L> {
        // ...
    }
}
