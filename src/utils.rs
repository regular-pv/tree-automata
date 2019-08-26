pub struct Combinations<'a, T, F, I, E: Clone> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    sources: &'a [T],
    f: F,
    iterators: Vec<I>,
    elements: Vec<E>,
    visited: bool
}

impl<'a, T, F, I, E: Clone> Combinations<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    pub fn new(sources: &'a [T], f: F) -> Combinations<'a, T, F, I, E> {
        let iterators = if sources.is_empty() {
            Vec::new()
        } else {
            vec![(f)(&sources[0])]
        };
        Combinations {
            sources: sources,
            f: f,
            iterators: iterators,
            elements: Vec::new(),
            visited: false
        }
    }
}

impl<'a, T, F, I, E: Clone> Iterator for Combinations<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    type Item = Vec<E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sources.is_empty() {
            if self.visited {
                None
            } else {
                self.visited = true;
                Some(Vec::new())
            }
        } else {
            while self.elements.len() < self.sources.len() {
                // let i = self.elements.len();
                // if i <= self.iterators.len() {
                //     self.iterators.push((self.f)(&self.sources[i]))
                // }

                match self.iterators.last_mut().unwrap().next() {
                    Some(e) => {
                        self.elements.push(e);
                        let j = self.elements.len();
                        if j < self.sources.len() {
                            self.iterators.push((self.f)(&self.sources[j]))
                        }
                    },
                    None => {
                        match self.elements.pop() {
                            Some(_) => {
                                self.iterators.pop();
                            },
                            None => return None
                        }
                    }
                }
            }

            let item = self.elements.clone();
            self.elements.pop();
            Some(item)
        }
    }
}

pub fn combinations<'a, T, F, I, E: Clone>(sources: &'a [T], f: F) -> Combinations<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    Combinations::new(sources, f)
}

pub struct CombinationsOption<'a, T, F, I, E: Clone> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    sources: &'a [T],
    f: F,
    iterators: Vec<(I, bool)>,
    elements: Vec<Option<E>>,
    visited: bool,
    weak: bool
}

impl<'a, T, F, I, E: Clone> CombinationsOption<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    pub fn new(sources: &'a [T], f: F, weak: bool) -> CombinationsOption<'a, T, F, I, E> {
        let iterators = if sources.is_empty() {
            Vec::new()
        } else {
            vec![((f)(&sources[0]), false)]
        };
        CombinationsOption {
            sources: sources,
            f: f,
            iterators: iterators,
            elements: Vec::new(),
            visited: false,
            weak: weak
        }
    }
}

impl<'a, T, F, I, E: Clone> Iterator for CombinationsOption<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    type Item = Vec<Option<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sources.is_empty() {
            if self.visited {
                None
            } else {
                self.visited = true;
                Some(Vec::new())
            }
        } else {
            while self.elements.len() < self.sources.len() {
                // assert!(self.iterators.len() <= self.sources.len());
                // for (_, visited) in self.iterators.iter() {
                //     print!("{}, ", visited);
                // }
                // print!(" for {} elements/ {}\n", self.elements.len(), self.sources.len());

                // let i = self.elements.len();
                // if self.iterators.len() <= i {
                //     println!("push iterator");
                //     self.iterators.push(((self.f)(&self.sources[i]), false))
                // }

                match self.iterators.last_mut() {
                    Some((ref mut it, ref mut visited)) => {
                        match it.next() {
                            Some(e) => {
                                self.elements.push(Some(e));
                                if !self.weak {
                                    *visited = true;
                                }
                                let j = self.elements.len();
                                if j < self.sources.len() {
                                    self.iterators.push(((self.f)(&self.sources[j]), false))
                                }
                            },
                            None => {
                                if *visited {
                                    // println!("visited");
                                    match self.elements.pop() {
                                        Some(_) => {
                                            self.iterators.pop();
                                        },
                                        None => {
                                            // println!("NONE");
                                            return None
                                        }
                                    }
                                } else {
                                    // println!("marked");
                                    self.elements.push(None);
                                    *visited = true;
                                    let j = self.elements.len();
                                    if j < self.sources.len() {
                                        self.iterators.push(((self.f)(&self.sources[j]), false))
                                    }
                                }
                            }
                        };
                    },
                    None => unreachable!()
                }
            }

            let item = self.elements.clone();
            self.elements.pop();
            Some(item)
        }
    }
}

pub fn combinations_option<'a, T, F, I, E: Clone>(sources: &'a [T], f: F) -> CombinationsOption<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    CombinationsOption::new(sources, f, false)
}

pub fn combinations_weak_option<'a, T, F, I, E: Clone>(sources: &'a [T], f: F) -> CombinationsOption<'a, T, F, I, E> where F: Fn(&'a T) -> I, I: 'a + Iterator<Item=E> {
    CombinationsOption::new(sources, f, true)
}

/// A multiplexer iterator that thats iterates on multiple iterators, alternating between iterators
/// at each iteration.
pub struct Mux<T: Iterator> {
    /// The iterators to iterate on.
    iterators: Vec<T>,

    /// The index of the next iterator to call `next` on.
    index: usize
}

impl<T: Iterator> Mux<T> {
    pub fn new(iterators: Vec<T>) -> Mux<T> {
        Mux {
            iterators: iterators,
            index: 0
        }
    }
}

impl<T: Iterator> Iterator for Mux<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        if self.iterators.is_empty() {
            None
        } else {
            let mut i = self.index;
            loop {
                if let Some(item) = self.iterators[i].next() {
                    self.index = (i + 1)%self.iterators.len();
                    return Some(item)
                } else {
                    i = (i + 1)%self.iterators.len();
                    if i == self.index {
                        // we rounded back on the same index without finding any item.
                        return None
                    }
                }
            }
        }
    }
}
