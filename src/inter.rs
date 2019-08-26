use std::fmt;
use crate::{Language, Symbol, State, Label, Labeled};
use crate::bottom_up::{Automaton, Configuration, CommonConfigurations};
use crate::combinations;

trait Inter<F>: Language<F> {
    type Output: Language<F>;

    fn inter(automata: &[&Self]) -> Self::Output;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Product<Q> {
    states: Vec<Q>
}

impl<Q: fmt::Display> fmt::Display for Product<Q> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.states.split_first() {
            Some((head, tail)) => {
                write!(f, "{}", head)?;
                for q in tail.iter() {
                    write!(f, "×{}", q)?;
                }
                Ok(())
            },
            None => Ok(())
        }
    }
}

impl<Q: Clone> Product<Q> {
    pub fn new(states: &[Q]) -> Product<Q> {
        Product {
            states: states.iter().map(|q| q.clone()).collect()
        }
    }

    fn configurations<'a, F: Symbol, L: Label>(&'a self, automata: &'a [&'a Automaton<F, Q, L>]) -> ProductConfigurations<'a, F, Q, L> where Q: State {
        ProductConfigurations {
            it: Automaton::common_configurations(automata, &self.states)
        }
    }
}

impl<Q> From<Vec<Q>> for Product<Q> {
    fn from(vec: Vec<Q>) -> Product<Q> {
        Product {
            states: vec
        }
    }
}

impl<Q: Clone> From<Vec<&Q>> for Product<Q> {
    fn from(vec: Vec<&Q>) -> Product<Q> {
        Product {
            states: vec.iter().map(|q| (*q).clone()).collect()
        }
    }
}

pub struct ProductConfigurations<'a, F: Symbol, Q: State, L: Label> {
    it: CommonConfigurations<'a, F, Q, L>
}

impl<'a, F: Symbol, Q: State, L: Label> Iterator for ProductConfigurations<'a, F, Q, L> {
    type Item = Labeled<Configuration<F, Product<Q>>, L>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it.next() {
            Some(ref confs) => {
                let (first, label) = confs.first().unwrap();
                let (f, arity) = first.signature();
                let mut states = Vec::with_capacity(arity);
                for i in 0..arity {
                    let mut product = Vec::with_capacity(confs.len());
                    for (conf, _) in confs {
                        product.push(conf.states()[i].clone());
                    }
                    states.push(product.into())
                }

                Some((Configuration(f.clone(), states), label.clone()))
            },
            None => None
        }
    }
}

impl<F: Symbol, Q: State, L: Label> Inter<F> for Automaton<F, Q, L> where Q: Clone {
    type Output = Automaton<F, Product<Q>, L>;

    fn inter(automata: &[&Self]) -> Automaton<F, Product<Q>, L> {
        fn process_state<F: Symbol, Q: State, L: Label>(automata: &[&Automaton<F, Q, L>], aut: &mut Automaton<F, Product<Q>, L>, product_state: Product<Q>) where Q: Clone {
            for (conf, label) in product_state.configurations(automata) {
                aut.add(conf.clone(), label, product_state.clone());

                for sub_state in conf.states() {
                    if !aut.includes(&sub_state) {
                        process_state(automata, aut, sub_state.clone())
                    }
                }
            }
        }

        let mut aut = Automaton::new();

        for final_states in combinations(automata, |a| a.final_states()) {
            let product = final_states.into();
            process_state(automata, &mut aut, product)
        }

        aut
    }
}
