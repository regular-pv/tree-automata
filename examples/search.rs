extern crate terms;
extern crate tree_automata as ta;

use terms::{Term, Pattern};
use ta::Automaton;

fn main() {
    let mut a = Automaton::new();

    a.add(Pattern::cons('a', &[]), (), "ab");
    a.add(Pattern::cons('b', &[]), (), "ab");
    a.add(Pattern::cons('b', &[]), (), "bc");
    a.add(Pattern::cons('c', &[]), (), "bc");

    assert!(a.configurations_for(&Pattern::cons('a', &[])).next().is_some());
    assert!(a.configurations_for(&Pattern::cons('a', &[])).next().is_some());
    assert!(a.configurations_for(&Pattern::cons('a', &[])).next().is_some());
    assert!(a.configurations_for(&Pattern::cons('a', &[])).next().is_some());

    a.add(Pattern::cons('i', &["ab".into()]), (), "i(ab)");
    a.add(Pattern::cons('j', &["ab".into()]), (), "j(ab)");
    a.add(Pattern::cons('k', &["ab".into()]), (), "k(ab)");
    a.add(Pattern::cons('l', &["ab".into()]), (), "l(ab)");
    a.add(Pattern::cons('l', &["bc".into()]), (), "l(bc)");
    a.add(Pattern::cons('m', &["bc".into()]), (), "m(bc)");
    a.add(Pattern::cons('n', &["bc".into()]), (), "n(bc)");
    a.add(Pattern::cons('o', &["bc".into()]), (), "o(bc)");

    a.add(Pattern::cons('f', &["i(ab)".into()]), (), "f(ij)|g(kl)");
    a.add(Pattern::cons('f', &["j(ab)".into()]), (), "f(ij)|g(kl)");
    a.add(Pattern::cons('g', &["k(ab)".into()]), (), "f(ij)|g(kl)");
    a.add(Pattern::cons('g', &["l(ab)".into()]), (), "f(ij)|g(kl)");
    a.add(Pattern::cons('g', &["l(bc)".into()]), (), "g(lm)|h(no)");
    a.add(Pattern::cons('g', &["m(bc)".into()]), (), "g(lm)|h(no)");
    a.add(Pattern::cons('h', &["n(bc)".into()]), (), "g(lm)|h(no)");
    a.add(Pattern::cons('h', &["o(bc)".into()]), (), "g(lm)|h(no)");

    let automata = [&a, &a];
    let positions = ["f(ij)|g(kl)".into(), "g(lm)|h(no)".into()];
    let patterns = [(), ()];
    let constraints = ();

    // g(l(b))
    let term = Term::new('g', &[Term::new('l', &[Term::new('b', &[])])]);

    let mut it = Automaton::synchronized_runs(
        &automata,
        &positions,
        &patterns,
        &constraints
    );

    // we should find term "g(l(b))"
    let mut empty = true;
    for (run, _) in it {
        empty = false;
        let t = run.first().unwrap();
        println!("term: {:?}", t);
        assert!(*t == term)
    }
    assert!(!empty);
}
