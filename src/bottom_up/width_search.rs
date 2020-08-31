use std::fmt;
use terms::Term;
use crate::{
	Symbol,
	State,
	NoLabel,
	bottom_up::{
		Automaton,
		Configuration,
		// Configurations
	}
};

#[derive(Debug, Clone, Copy)]
pub struct Killed;

pub trait LanguageState<F: Symbol, E: ?Sized>: State + Sized {
	fn configurations<'a>(&self, env: &'a E) -> Box<dyn Iterator<Item = Configuration<F, Self>> + 'a>;
}

impl<F: Symbol, Q: State> LanguageState<F, Automaton<F, Q, NoLabel>> for Q {
	fn configurations<'a>(&self, aut: &'a Automaton<F, Q, NoLabel>) -> Box<dyn Iterator<Item = Configuration<F, Self>> + 'a> {
		Box::new(aut.configurations_for_state(self).map(|(conf, _)| conf.clone()))
	}
}

impl<'e, F: Symbol, Q: State> LanguageState<F, [&'e Automaton<F, Q, NoLabel>]> for Indexed<Q> {
	fn configurations<'a>(&self, automata: &'a [&'e Automaton<F, Q, NoLabel>]) -> Box<dyn Iterator<Item = Configuration<F, Self>> + 'a> {
		let index = self.1;
		let aut = automata[index];
		Box::new(aut.configurations_for_state(&self.0).map(move |(conf, _)| {
			let states = conf.states().iter().map(|q| Indexed((*q).clone(), index)).collect();
			Configuration(conf.symbol().clone(), states)
		}))
	}
}

fn add_language_state<'a, F: Symbol, E, Q: LanguageState<F, E>>(q: Q, env: &'a E, aut: &mut Automaton<F, Q, NoLabel>) {
	if !aut.includes(&q) {
		for conf in q.configurations(env) {
			aut.add(conf.clone(), NoLabel, q.clone());
			for sub in conf.1.into_iter() {
				add_language_state(sub, env, aut);
			}
		}
	}
}

impl<'a, F: Symbol, E, Q: LanguageState<F, E>> From<(Q, &'a E)> for Automaton<F, Q, NoLabel> {
	fn from((q, env): (Q, &'a E)) -> Automaton<F, Q, NoLabel> {
		let mut aut = Automaton::new();
		add_language_state(q.clone(), env, &mut aut);
		aut.set_final(q);
		aut
	}
}

impl<'a, F: Symbol, Q: LanguageState<F, ()>> From<Q> for Automaton<F, Q, NoLabel> {
	fn from(q: Q) -> Automaton<F, Q, NoLabel> {
		let mut aut = Automaton::new();
		add_language_state(q.clone(), &(), &mut aut);
		aut.set_final(q);
		aut
	}
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Indexed<Q: State>(pub Q, pub usize);

impl<T: State + fmt::Display> fmt::Display for Indexed<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}.{}", self.0, self.1)
	}
}

#[cfg(not(debug_assertions))]
pub trait SearchPattern<F: Symbol, Q: State, C: SearchContext>: Clone {
	fn matches(&self, depth: usize, context: &C, q: &Q, configuration: &Configuration<F, Q>) -> Option<(C, Vec<Self>)>;
}

#[cfg(debug_assertions)]
pub trait SearchPattern<F: Symbol, Q: State, C: SearchContext>: Clone + fmt::Display + fmt::Debug {
	fn matches(&self, depth: usize, context: &C, q: &Q, configuration: &Configuration<F, Q>) -> Option<(C, Vec<Self>)>;
}

#[cfg(not(debug_assertions))]
pub trait SearchContext: Clone {
	fn looping(&self) -> bool;
}

#[cfg(debug_assertions)]
pub trait SearchContext: Clone + fmt::Display + fmt::Debug {
	fn looping(&self) -> bool;
}

#[cfg(debug_assertions)]
struct DebugInfos {
	depth: usize
}

#[derive(Hash, PartialEq, Eq)]
struct Item<F: Symbol, Q: State, C, P>(Configuration<F, Q>, C, Vec<P>);

impl<F: Symbol + fmt::Display, Q: State + fmt::Display, C: fmt::Display, P: fmt::Display> fmt::Debug for Item<F, Q, C, P> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}/{}/{})", self.0, self.1, PList(&self.2))
	}
}

/// A term fragment provides an iterator over the terms satisfying a given context.
/// It performs a width-first search into the given automata.
pub struct TermFragment<'a, E: ?Sized, F: Symbol, Q: LanguageState<F, E>, C: SearchContext, P: SearchPattern<F, Q, C>> {
	env: &'a E,

	/// Context
	context: C,

	/// Patterns
	patterns: Vec<P>,

	/// The leaves of the term fragment.
	leaves: Vec<Q>,

	/// Possible configurations for the leaves
	leaves_configurations: Vec<Box<dyn Iterator<Item = Configuration<F, Q>> + 'a>>,

	/// Current configurations.
	current: Vec<Item<F, Q, C, P>>,

	/// Current configurations, and next fragment.
	next_fragment: Option<Box<TermFragment<'a, E, F, Q, C, P>>>,

	/// Has it been visited yet? (has the `next` method been called at least once?)
	/// This is important for the empty fragment. The `next` method must then return
	/// Some(Vec::new()) this first time it is called, and None the rest of the time.
	visited: bool,

	/// The depth of the fragment.
	depth: usize,

	kill_signal: Option<crossbeam_channel::Receiver<()>>
}

impl<'a, E: ?Sized, F: Symbol, Q: LanguageState<F, E>, C: SearchContext, P: SearchPattern<F, Q, C>> TermFragment<'a, E, F, Q, C, P> {
	pub fn new(env: &'a E, depth: usize, context: C, leaves: Vec<Q>, patterns: Vec<P>, kill_signal: Option<crossbeam_channel::Receiver<()>>) -> TermFragment<'a, E, F, Q, C, P> {
		let leaves_configurations = if leaves.is_empty() {
			Vec::new()
		} else {
			vec![leaves[0].configurations(env)]
		};
		TermFragment {
			env,
			context,
			patterns,
			leaves,
			leaves_configurations,
			current: Vec::new(),
			next_fragment: None,
			visited: false,
			depth,
			kill_signal
		}
	}
}

impl<'a, E: ?Sized, F: Symbol, Q: LanguageState<F, E>, C: SearchContext, P: SearchPattern<F, Q, C>> Iterator for TermFragment<'a, E, F, Q, C, P> {
	type Item = Result<Vec<Term<F>>, Killed>;

	fn next(&mut self) -> Option<Result<Vec<Term<F>>, Killed>> {
		// #[cfg(debug_assertions)]
		// println!("next: {}", self);

		loop {
			// Check if the search is canceled.
			if let Some(kill_signal) = self.kill_signal.as_ref() {
				if let Ok(()) = kill_signal.try_recv() {
					return Some(Err(Killed))
				}
			}

			if self.leaves.is_empty() {
				if self.visited {
					return None
				} else {
					self.visited = true;
					return Some(Ok(Vec::new()))
				}
			} else {
				let current_context = match self.current.last() {
					Some(Item(_, context, _)) => context,
					None => &self.context
				};

				if self.current.len() < self.leaves.len() {
					let i = self.current.len();
					let j = i + 1;

					match self.leaves_configurations.last_mut().unwrap().next() {
						Some(conf) => {
							if let Some((next_context, sub_patterns)) = self.patterns[i].matches(self.depth, &current_context, &self.leaves[i], &conf) {
								assert!(conf.states().len() == sub_patterns.len());
								self.current.push(Item(conf, next_context, sub_patterns));

								// println!("accepted: {:?}", self.current);

								if j < self.leaves.len() {
									let confs = self.leaves[j].configurations(self.env);
									self.leaves_configurations.push(confs);
								}
							} else {
								// println!("rejected: {:?} -- {}", self.current, conf)
							}
						},
						None => {
							if i > 0 {
								self.current.pop();
								self.leaves_configurations.pop();
							} else {
								return None
							}
						}
					}
				} else {
					match self.next_fragment {
						Some(ref mut frag) => {
							match frag.next() {
								Some(Err(Killed)) => return Some(Err(Killed)),
								Some(Ok(mut sub_terms)) => {
									sub_terms.reverse();
									let mut terms = Vec::with_capacity(self.current.len());

									for Item(Configuration(f, sub_states), _, _) in self.current.iter() {
										let mut subs = Vec::with_capacity(sub_states.len());
										for _ in sub_states.iter() {
											subs.push(sub_terms.pop().unwrap());
										}

										terms.push(Term::new(f.clone(), subs))
									}

									return Some(Ok(terms))
								},
								None => {
									// println!("not sub fragment.");
									self.next_fragment = None;
									self.current.pop();
								}
							}
						},
						None if !current_context.looping() => {
							let mut leaves = Vec::new();
							let mut patterns = Vec::new();
							for Item(Configuration(_, sub_states), _, sub_patterns) in self.current.iter() {
								for q in sub_states.iter() {
									leaves.push(q.clone())
								}

								for p in sub_patterns.iter() {
									patterns.push(p.clone())
								}
							}

							let next_fragment = TermFragment::new(self.env, self.depth + 1, current_context.clone(), leaves, patterns, self.kill_signal.clone());
							self.next_fragment = Some(Box::new(next_fragment))
						},
						None => { // loop detected
							self.current.pop();
						}
					}
				}
			}
		}
	}
}

struct PList<'a, T: fmt::Display>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for PList<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.0.split_first() {
			Some((head, tail)) => {
				head.fmt(f)?;
				for e in tail.iter() {
					write!(f, ",")?;
					e.fmt(f)?;
				}
				Ok(())
			},
			None => Ok(())
		}
	}
}

#[cfg(debug_assertions)]
impl<'a, E: ?Sized, F: Symbol, Q: LanguageState<F, E>, C: SearchContext, P: SearchPattern<F, Q, C>> fmt::Display for TermFragment<'a, E, F, Q, C, P> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}) {} with context {}", PList(&self.patterns), PList(&self.leaves), self.context)
	}
}
