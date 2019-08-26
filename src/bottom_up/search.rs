use std::rc::Rc;
use terms::Term;
use crate::{Symbol, State, Label, Labeled};
use crate::utils::combinations;
use super::{Automaton, Configuration, Configurations};

/// Common configurations searcher.
pub struct CommonConfigurations<'a, F: Symbol, Q: State, L: Label> {
    automata: &'a [&'a Automaton<F, Q, L>],
    positions: &'a [Q],
    iterators: Vec<Configurations<'a, F, Q, L>>,
    configurations: Vec<Labeled<Configuration<F, Q>, L>>,
    // pattern: P
}

impl<'a, F: Symbol, Q: State, L: Label> CommonConfigurations<'a, F, Q, L> {
    pub fn new(
        automata: &'a [&'a Automaton<F, Q, L>],
        positions: &'a [Q]
    ) -> CommonConfigurations<'a, F, Q, L>
    {
        // println!("looking for common confs: {:?} {:?}", positions, patterns);

        let mut iterators = Vec::with_capacity(positions.len());
        if !positions.is_empty() {
            iterators.push(automata[0].configurations_for_state(&positions[0]))
        };

        CommonConfigurations {
            automata: automata,
            positions: positions,
            iterators: iterators,
            configurations: Vec::with_capacity(positions.len())
        }
    }
}

impl<'a, F: Symbol, Q: State, L: Label> Iterator for CommonConfigurations<'a, F, Q, L> {
    type Item = Vec<Labeled<Configuration<F, Q>, L>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.positions.is_empty() {
            None
        } else {
            while self.configurations.len() < self.positions.len() {
                let i = self.configurations.len();
                let j = i + 1;

                match self.iterators.last_mut().unwrap().next() {
                    Some((Configuration(f, sub_states), label)) => {
                        if self.configurations.is_empty() || f == self.configurations.first().unwrap().0.symbol() {
                            if j < self.positions.len() {
                                let it = self.automata[j].configurations_for_state(&self.positions[j]);
                                // println!("new it");
                                self.iterators.push(it);
                            }
                            self.configurations.push((Configuration(f.clone(), sub_states.clone()), label.clone()))
                        }
                    },
                    None => {
                        match self.configurations.pop() {
                            Some(_) => {
                                if i < self.positions.len() {
                                    // println!("drop it");
                                    self.iterators.pop();
                                }
                            },
                            None => {
                                // println!("no other common confs.");
                                return None
                            }
                        }
                    }
                }
            }

            let result = self.configurations.clone();
            self.configurations.pop();
            Some(result)
        }
    }
}

#[derive(Clone)]
struct VisitedTransitions<'a, F: Symbol, Q: State, L: Label> {
    previously_visited: Option<Rc<VisitedTransitions<'a, F, Q, L>>>,
    conf: &'a Configuration<F, Q>,
    label: &'a L,
    q: &'a Q
}

impl<'a, F: Symbol, Q: State, L: Label> VisitedTransitions<'a, F, Q, L> {
    fn new(previously_visited: &Option<Rc<VisitedTransitions<'a, F, Q, L>>>, conf: &'a Configuration<F, Q>, label: &'a L, q: &'a Q) -> VisitedTransitions<'a, F, Q, L> {
        VisitedTransitions {
            previously_visited: previously_visited.clone(),
            conf: conf,
            label: label,
            q: q
        }
    }

    fn contains(&self, conf: &'a Configuration<F, Q>, label: &'a L, q: &'a Q) -> bool {
        // println!("compare {:?} -{:?}-> {:?} and {:?} -{:?}-> {:?}", self.conf, self.label, self.q, conf, label, q);
        if self.q == q && self.label == label && self.conf == conf {
            // println!("visited");
            true
        } else {
            match &self.previously_visited {
                Some(previously_visited) => previously_visited.contains(conf, label, q),
                None => false
            }
        }
    }
}

pub struct Representatives<'a, F: Symbol, Q: State, L: Label> {
    automaton: &'a Automaton<F, Q, L>,
    visited_transitions: Option<Rc<VisitedTransitions<'a, F, Q, L>>>,
    pending_states: Vec<&'a Q>,
    current_state: Option<(&'a Q, Configurations<'a, F, Q, L>)>,
    current_configuration: Option<(&'a F, Box<dyn Iterator<Item=Vec<Term<F>>> + 'a>)>
}

impl<'a, F: Symbol, Q: State, L: Label> Representatives<'a, F, Q, L> {
    pub fn new(aut: &'a Automaton<F, Q, L>) -> Representatives<'a, F, Q, L> {
        let mut pending_states: Vec<&'a Q> = aut.final_states.iter().collect();
        let current_state = match pending_states.pop() {
            Some(q) => Some((q, aut.configurations_for_state(q))),
            None => None
        };
        Representatives {
            automaton: aut,
            visited_transitions: None,
            pending_states: pending_states,
            current_state: current_state,
            current_configuration: None
        }
    }

    pub fn visited(&self, conf: &'a Configuration<F, Q>, label: &'a L, q: &'a Q) -> bool {
        match &self.visited_transitions {
            Some(visited_transitions) => visited_transitions.contains(conf, label, q),
            None => false
        }
    }
}

impl<'a, F: Symbol, Q: State, L: Label> Iterator for Representatives<'a, F, Q, L> {
    type Item = Term<F>;

    fn next(&mut self) -> Option<Term<F>> {
        loop {
            match self.current_configuration {
                Some((f, ref mut iterator)) => {
                    match iterator.next() {
                        Some(sub_terms) => {
                            return Some(Term::new(f.clone(), sub_terms))
                        },
                        None => self.current_configuration = None
                    }
                },
                None => {
                    match self.current_state {
                        Some((q, ref mut configurations)) => {
                            match configurations.next() {
                                Some((conf, label)) => {
                                    if !self.visited(conf, label, q) {
                                        let aut = self.automaton;
                                        let visited_transitions = Some(Rc::new(VisitedTransitions::new(&self.visited_transitions, conf, label, q)));
                                        let sub_terms_it = combinations(conf.states(), move |q| Representatives {
                                            automaton: aut,
                                            visited_transitions: visited_transitions.clone(),
                                            pending_states: Vec::new(),
                                            current_state: Some((q, aut.configurations_for_state(q))),
                                            current_configuration: None
                                        });
                                        // println!("conf: {:?}", conf);
                                        self.current_configuration = Some((conf.symbol(), Box::new(sub_terms_it)));
                                    }
                                },
                                None => self.current_state = None
                            }
                        },
                        None => {
                            match self.pending_states.pop() {
                                Some(q) => {
                                    self.current_state = Some((q, self.automaton.configurations_for_state(q)))
                                },
                                None => return None
                            }
                        }
                    }
                }
            }
        }
    }
}
