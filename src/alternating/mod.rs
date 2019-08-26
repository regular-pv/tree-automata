use std::collections::{HashMap, HashSet};
use crate::{Symbol, Label, State};
use crate::bottom_up;

/// The empty conjunction is True.
pub type Conjuction<Q, I> = Vec<(I, Q)>;

/// The empty clause is False.
pub type Clause<Q, I> = Vec<Conjuction<Q, I>>;

/// Alternating tree automaton.
pub struct Automaton<F: Symbol, Q: State, I> {
    dummy_clauses: HashMap<F, Clause<Q, I>>,

    /// Initial states.
    initial_states: HashSet<Q>,

    /// Internal structure of the automaton.
    state_clauses: HashMap<Q, HashMap<F, Clause<Q, I>>>
}

impl<F: Symbol, Q: State, I> Automaton<F, Q, I> {
    /// Create a new empty alternating tree automaton.
    pub fn new() -> Automaton<F, Q, I> {
        Automaton {
            dummy_clauses: HashMap::new(),
            initial_states: HashSet::new(),
            state_clauses: HashMap::new()
        }
    }

    pub fn states(&self) -> std::collections::hash_map::Keys<Q, HashMap<F, Clause<Q, I>>> {
        self.state_clauses.keys()
    }

    pub fn clauses_for_state(&self, q: &Q) -> std::collections::hash_map::Iter<F, Clause<Q, I>> {
        match self.state_clauses.get(q) {
            Some(clauses) => clauses.iter(),
            None => self.dummy_clauses.iter()
        }
    }

    /// Add the given conjuction to the clause (state, symbol).
    pub fn add(&mut self, state: &Q, symbol: &F, conjuction: Conjuction<Q, I>) {
        match self.state_clauses.get_mut(state) {
            Some(ref mut symbol_clauses) => {
                match symbol_clauses.get_mut(symbol) {
                    Some(ref mut clause) => {
                        clause.push(conjuction);
                    },
                    None => {
                        let clause = vec![conjuction];
                        symbol_clauses.insert(symbol.clone(), clause);
                    }
                }
            },
            None => {
                let mut symbol_clauses = HashMap::new();
                let clause = vec![conjuction];
                symbol_clauses.insert(symbol.clone(), clause);
                self.state_clauses.insert(state.clone(), symbol_clauses);
            }
        }
    }

    pub fn is_initial(&self, q: &Q) -> bool {
        self.initial_states.contains(q)
    }

    /// Set the given state an initial state.
    /// Return `true` if the state was not already initial.
    /// Return `false` if it was already an initial state.
    pub fn set_initial(&mut self, q: Q) -> bool {
        self.initial_states.insert(q)
    }

    pub fn map_states<R: State, M>(&self, g: M) -> Automaton<F, R, I> where M: Fn(&Q) -> R, I: Clone {
        let mut mapped = Automaton::new();

        for (state, clauses) in self.state_clauses.iter() {
            let state = g(state);

            for (f, clause) in clauses.iter() {
                for conjunction in clause.iter() {
                    let mapped_conjunction = conjunction.iter().map(|(index, q)| {
                        (index.clone(), g(q))
                    }).collect();

                    mapped.add(&state, f, mapped_conjunction);
                }
            }
        }

        for q in self.initial_states.iter() {
            mapped.set_initial(g(q));
        }

        mapped
    }
}

impl<F: Symbol, Q: State> Automaton<F, Q, u32> {
    /// Add a bottom-up transition.
    /// It is added as a clause to the corresponding (state, symbol) pair.
    pub fn add_transition(&mut self, bottom_up::Configuration(f, states): &bottom_up::Configuration<F, Q>, state: &Q) {
        let conjunction = states.iter().enumerate().map(|(i, q)| (i as u32, q.clone())).collect();
        self.add(state, f, conjunction)
    }
}

impl<'a, F: Symbol, Q: State, L: Label> From<&'a bottom_up::Automaton<F, Q, L>> for Automaton<F, Q, u32> {
    fn from(bottom_up: &'a bottom_up::Automaton<F, Q, L>) -> Automaton<F, Q, u32> {
        let mut alternating = Automaton::new();

        // add all transitions.
        for (conf, _, q) in bottom_up.transitions() {
            alternating.add_transition(conf, q)
        }

        // set intial states.
        for q in bottom_up.final_states() {
            alternating.set_initial(q.clone());
        }

        alternating
    }
}
