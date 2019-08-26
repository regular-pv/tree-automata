use std::fmt;
use std::collections::{hash_set, hash_map, HashSet, HashMap};
use terms::{
    PatternLike,
    PatternLikeKind
};

use crate::{
    utils::combinations,
    Symbol,
    State,
    Label,
    Ranked,
    NoLabel,
    Labeled,
    Language,
    //LanguageState,
    ConfigurationIterator
};

pub mod macros;
pub mod search;
pub mod width_search;

pub use search::*;
pub use width_search::*;

/// Tree automaton configuration.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Configuration<F, Q: State>(pub F, pub Vec<Q>);

impl<F, Q: State> Configuration<F, Q> {
    pub fn symbol(&self) -> &F {
        &self.0
    }

    pub fn states(&self) -> &[Q] {
        &self.1
    }

    pub fn len(&self) -> usize {
        self.1.len()
    }

    pub fn signature(&self) -> (&F, usize) {
        (&self.0, self.1.len())
    }

    pub fn map<R: State, M>(&self, g: M) -> Configuration<F, R> where M: Fn(&Q) -> R, F: Clone {
        let states = self.1.iter().map(|q| g(q)).collect();
        Configuration(self.0.clone(), states)
    }
}

impl<F: fmt::Display, Q: State + fmt::Display> fmt::Display for Configuration<F, Q> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)?;
        match self.1.split_first() {
            Some((head, tail)) => {
                write!(f, "({}", head)?;
                for e in tail.iter() {
                    write!(f, ", {}", e)?;
                }
                write!(f, ")")
            },
            None => Ok(())
        }
    }
}

pub type Configurations<'a, F, Q, L> = hash_set::Iter<'a, Labeled<Configuration<F, Q>, L>>;

pub struct Transifions<'a, F, Q: State, L: Label> {
    it: hash_map::Iter<'a, Q, HashSet<Labeled<Configuration<F, Q>, L>>>,
    current: Option<(&'a Q, Configurations<'a, F, Q, L>)>
}

impl<'a, F, Q: State, L: Label> Transifions<'a, F, Q, L> {
    fn new(aut: &'a Automaton<F, Q, L>) -> Transifions<'a, F, Q, L> {
        Transifions {
            it: aut.state_configurations.iter(),
            current: None
        }
    }
}

impl<'a, F, Q: State, L: Label> Iterator for Transifions<'a, F, Q, L> {
    type Item = (&'a Configuration<F, Q>, &'a L, &'a Q);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current {
                Some((q, ref mut configurations)) => {
                    match configurations.next() {
                        Some((configuration, label)) => {
                            return Some((configuration, label, q))
                        },
                        None => self.current = None
                    }
                },
                None => {
                    match self.it.next() {
                        Some((q, configurations)) => self.current = Some((q, configurations.iter())),
                        None => return None
                    }
                }
            }
        }
    }
}

/// Tree automaton.
#[derive(Clone)]
pub struct Automaton<F, Q: State, L: Label> {
    /// An empty set of labeled states used to return an empty iterator.
    dummy_states: HashSet<Labeled<Q, L>>,

    /// An empty set of labeled configurations used to return an empty iterator.
    dummy_configurations: HashSet<Labeled<Configuration<F, Q>, L>>,

    /// For each configuration t, associates the set of states such that t -> q.
    configuration_states: HashMap<Configuration<F, Q>, HashSet<Labeled<Q, L>>>,

    /// For each state q, associates the set of configurations such that t -> q.
    state_configurations: HashMap<Q, HashSet<Labeled<Configuration<F, Q>, L>>>,

    // /// For each configuration, associate every known labeled version.
    // labeled_configurations: HashMap<Configuration<F, Q>, HashSet<Labeled<Configuration<F, Q>, L>>>,

    /// Final states of the automaton.
    final_states: HashSet<Q>
}

pub type States<'a, F, Q, L> = hash_map::Keys<'a, Q, HashSet<Labeled<Configuration<F, Q>, L>>>;

impl<F: Symbol, Q: State, L: Label> Language<F> for Automaton<F, Q, L> {
    // ...
}

impl<F: Symbol, Q: State, L: Label> Automaton<F, Q, L> {
    // type Configuration = Configuration<F, Q>;

    /// Create a new empty automaton.
    pub fn new() -> Automaton<F, Q, L> {
        Automaton {
            dummy_states: HashSet::new(),
            dummy_configurations: HashSet::new(),
            configuration_states: HashMap::new(),
            state_configurations: HashMap::new(),
            // labeled_configurations: HashMap::new(),
            final_states: HashSet::new()
        }
    }

    /// Return the number of states in the automaton.
    pub fn len(&self) -> usize {
        self.state_configurations.len()
    }

    pub fn states(&self) -> States<F, Q, L> {
        self.state_configurations.keys()
    }

    /// Return an iterator to the final states of the automaton.
    pub fn final_states(&self) -> hash_set::Iter<Q> {
        self.final_states.iter()
    }

    /// Set the given state a final state.
    /// Return `true` if the state was not already final.
    /// Return `false` if the state was already a final state.
    pub fn set_final(&mut self, q: Q) -> bool {
        self.final_states.insert(q)
    }

    /// Checks if the given state is in the automaton.
    /// Return true if at least one configuration is attached to the state in the automaton.
    pub fn includes(&self, q: &Q) -> bool {
        self.state_configurations.get(q).is_some()
    }

    pub fn transitions(&self) -> Transifions<F, Q, L> {
        Transifions::new(self)
    }

    /// Return an iterator over the configurations connected to the given state.
    pub fn configurations_for_state(&self, q: &Q) -> Configurations<F, Q, L> {
        match self.state_configurations.get(q) {
            Some(confs) => confs.iter(),
            None => self.dummy_configurations.iter()
        }
    }

    // /// Return an iterator over the configurations compatible with the given configuration.
    // pub fn configurations_for<'a>(&'a self, source: &'a Configuration<F, Q>) -> Configurations<F, Q, L> {
    //     // match source.kind() {
    //     //     PatternKind::Var(q) => self.configurations_for_state(q),
    //     //     _ =>
    //     // }
    //     self.labeled_configurations(source)
    // }

    // /// Return an iterator over the known labeled versions of the given configuration.
    // pub fn labeled_configurations(&self, conf: &Configuration<F, Q>) -> Configurations<F, Q, L> {
    //     match self.labeled_configurations.get(conf) {
    //         Some(confs) => confs.iter(),
    //         None => self.dummy_configurations.iter()
    //     }
    // }

    /// Return an iterator over the states connected to the given configuration.
    pub fn states_for_configuration(&self, conf: &Configuration<F, Q>) -> hash_set::Iter<Labeled<Q, L>> {
        match self.configuration_states.get(conf) {
            Some(states) => states.iter(),
            None => self.dummy_states.iter()
        }
    }

    /// Add a new transition to the automaton.
    pub fn add(&mut self, conf: Configuration<F, Q>, label: L, state: Q) {
        match self.configuration_states.get_mut(&conf) {
            Some(states) => {
                states.insert((state.clone(), label.clone()));
            },
            None => {
                let mut states = HashSet::new();
                states.insert((state.clone(), label.clone()));
                self.configuration_states.insert(conf.clone(), states);
            }
        }

        // match self.labeled_configurations.get_mut(&conf) {
        //     Some(labeled_confs) => {
        //         labeled_confs.insert((conf.clone(), label.clone()));
        //     },
        //     None => {
        //         let mut labeled_confs = HashSet::new();
        //         labeled_confs.insert((conf.clone(), label.clone()));
        //         self.labeled_configurations.insert(conf.clone(), labeled_confs);
        //     }
        // }

        match self.state_configurations.get_mut(&state) {
            Some(configurations) => {
                configurations.insert((conf, label));
            },
            None => {
                let mut configurations = HashSet::new();
                configurations.insert((conf, label));
                self.state_configurations.insert(state, configurations);
            }
        }
    }

    /// Add new transitions in the automaton by adding and normalizing the given configuration,
    /// label and state.
    pub fn add_normalized<P: PatternLike<F, Q>, N>(&mut self, pattern: &P, normalizer: &mut N) -> Q
    where N: FnMut(&Configuration<F, Q>) -> Labeled<Q, L> {
        match pattern.kind() {
            PatternLikeKind::Cons(f, l) => {
                let mut normalized_l = Vec::with_capacity(l.len());
                for sub_conf in l.iter() {
                    let q = match sub_conf.kind() {
                        PatternLikeKind::Var(q) => q.clone(),
                        _ => {
                            self.add_normalized(sub_conf, normalizer)
                        }
                    };
                    normalized_l.push(q);
                }

                let normalized_conf = Configuration(f.clone(), normalized_l);

                if let Some((state, _)) = self.states_for_configuration(&normalized_conf).next() {
                    state.clone()
                } else {
                    let (state, label) = (*normalizer)(&normalized_conf);
                    self.add(normalized_conf, label, state.clone());
                    state
                }
            },
            PatternLikeKind::Var(_) => {
                panic!("epsilon transitions are not allowed!")
            }
        }
    }

    // /// Find a run in the automaton that recognizes the given pattern.
    // pub fn find<X>(&self, pattern: Pattern<F, X>) -> Option<Configuration<F, Labeled<Q, L>>> {
    //     panic!("TODO")
    // }

    /// Automata common configurations.
    pub fn common_configurations<'a>(
        automata: &'a [&'a Automaton<F, Q, L>],
        positions: &'a [Q]
    ) -> CommonConfigurations<'a, F, Q, L> {
        CommonConfigurations::new(automata, positions)
    }

    // / Return an iterator over all the synchronized runs between the given automata,
    // / starting from the given positions, matching the given patterns and with the given
    // / constraints.
    // pub fn synchronized_runs<'a, P: pattern::Meta<F>>(
    //     automata: &'a [&'a Automaton<F, Q, L>],
    //     positions: &'a [Configuration<F, Q>],
    //     patterns: &'a [P],
    //     constraints: &'a P::Constraints
    // ) -> SynchronizedRuns<'a, F, Q, L, P>
    // where P: Clone, P::Constraints: Clone
    // {
    //     SynchronizedRuns::new(automata, positions, patterns, constraints)
    // }

    /// Return an iterator over the representative terms of the automaton.
    /// The representatives terms are all the terms recognized by the automaton *without cycle*.
    /// Together they trigger every transition of the automaton.
    pub fn representatives(&self) -> Representatives<F, Q, L> {
        Representatives::new(self)
    }

    /// Complement the automaton.
    /// This will invert the set of final and non-final states.
    /// If the automaton is complete, then `self` becomes its own complement.
    pub fn complement(&mut self) {
        let states: HashSet<Q> = self.states().cloned().collect();
        let new_final_states = states.difference(&self.final_states).cloned().collect();
        self.final_states = new_final_states;
    }

    /// Return the alphabet on which this automaton is defined.
    pub fn alphabet(&self) -> HashSet<F> {
        let mut alphabet = HashSet::new();
        for (Configuration(f, _), _, _) in self.transitions() {
            alphabet.insert(f.clone());
        }

        alphabet
    }

    pub fn map_states<R: State, M>(&self, g: M) -> Automaton<F, R, L> where M: Fn(&Q) -> R {
        let mut configuration_states: HashMap<Configuration<F, R>, HashSet<Labeled<R, L>>> = HashMap::new();
        for (conf, states) in self.configuration_states.iter() {
            let conf = conf.map(|q| g(q));
            let states: HashSet<Labeled<R, L>> = states.iter().map(|(q, l)| (g(q), l.clone())).collect();
            match configuration_states.get_mut(&conf) {
                Some(conf_states) => {
                    conf_states.extend(states.into_iter());
                },
                None => {
                    configuration_states.insert(conf, states);
                }
            }
        }

        let mut state_configurations: HashMap<R, HashSet<Labeled<Configuration<F, R>, L>>> = HashMap::new();
        for (state, confs) in self.state_configurations.iter() {
            let state = g(state);
            let confs: HashSet<Labeled<Configuration<F, R>, L>> = confs.iter().map(|(conf, l)| (conf.map(|q| g(q)), l.clone())).collect();
            match state_configurations.get_mut(&state) {
                Some(state_confs) => {
                    state_confs.extend(confs.into_iter());
                },
                None => {
                    state_configurations.insert(state, confs);
                }
            }
        }

        let mut final_states = HashSet::new();
        for q in self.final_states.iter() {
            final_states.insert(g(q));
        }

        Automaton {
            dummy_states: HashSet::new(),
            dummy_configurations: HashSet::new(),
            configuration_states: configuration_states,
            state_configurations: state_configurations,
            final_states: final_states
        }
    }
}

impl<F: Symbol, Q: State> Automaton<F, Q, NoLabel> {
    /// Complete the language with the given automaton.
    /// Each state of `self` must be mappable into a state of `lang`, and each state of `lang`
    /// must be transformed into a dead state of `self`.
    pub fn complete_with<'a, A: Iterator<Item=&'a F>, R: State>(&mut self, alphabet: A, lang: &Automaton<F, R, NoLabel>) where F: 'a + Ranked, R: From<Q>, Q: From<R> {
        let mut states: Vec<Q> = self.states().map(|q| (*q).clone()).collect();
        states.extend(lang.states().map(|r| r.clone().into()));

        for f in alphabet {
            let indexes: Vec<usize> = (0..f.arity()).collect();
            for states in combinations(&indexes, |_| states.clone().into_iter()) {
                let conf = Configuration(f.clone(), states.clone());
                match self.states_for_configuration(&conf).next() {
                    Some(_) => (),
                    None => {
                        let parent_states: Vec<R> = states.iter().map(|q| (*q).clone().into()).collect();
                        if let Some((r, _)) = lang.states_for_configuration(&Configuration(f.clone(), parent_states)).next() {
                            self.add(conf, NoLabel, (*r).clone().into());
                        }
                    }
                }
            }
        }
    }
}

// fn map<'a, F: 'a + Clone, Q: 'a + State, L: 'a + Label>(conf: &'a Labeled<Configuration<F, Q>, L>) -> Labeled<Configuration<F, Q>, L> {
//     conf.clone()
// }

pub struct CloneConfigurations<'a, F: Symbol, Q: State, L: Label> {
    it: Configurations<'a, F, Q, L>
}

impl<'a, F: Symbol, Q: State, L: Label> Iterator for CloneConfigurations<'a, F, Q, L> {
    type Item = Configuration<F, Q>;

    fn next(&mut self) -> Option<Configuration<F, Q>> {
        match self.it.next() {
            Some((conf, _)) => Some(conf.clone()),
            None => None
        }
    }
}

impl<'a, F: Symbol, Q: State, L: Label> ConfigurationIterator<'a, F, Q> for CloneConfigurations<'a, F, Q, L> {
    // ...
}

// impl<F: Symbol, Q: State, L: Label> LanguageState<F, Automaton<F, Q, L>> for Q {
//     // type Configurations<'a> = std::iter::Map<Configurations<'a, F, Q, L>, fn(&'a Labeled<Configuration<F, Q>, L>) -> Configuration<F, Q>> where L: 'a;
//
//     fn configurations<'a>(&'a self, lang: &'a Automaton<F, Q, L>) -> Box<ConfigurationIterator<'a, F, Q> + 'a> {
//         Box::new(CloneConfigurations {
//             it: lang.configurations_for_state(self)
//         })
//     }
// }
//

impl<F: Symbol + fmt::Display, Q: State + fmt::Display, L: Label + fmt::Display> fmt::Display for Automaton<F, Q, L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (conf, label, q) in self.transitions() {
            write!(f, "{} -{}-> {}\n", conf, label, q)?;
        }
        write!(f, "final states: ")?;
        for q in self.final_states() {
            write!(f, "{} ", q)?;
        }
        Ok(())
    }
}
