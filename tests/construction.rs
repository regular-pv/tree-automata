extern crate terms;
extern crate tree_automata as ta;

use terms::Pattern;
use ta::Automaton;

#[test]
fn add_simple_configurations() {
    let mut a = Automaton::new();
    let conf = Pattern::cons((), &[]);

    assert!(a.states_for_configuration(&conf).next().is_none());
    a.add(conf.clone(), (), ());
    assert!(*a.states_for_configuration(&conf).next().unwrap() == ((), ()))
}

#[test]
fn add_multiple_configurations() {
    let mut a = Automaton::new();

    let confs = [
        Pattern::cons("f", &[]),
        Pattern::cons("g", &[]),
        Pattern::cons("h", &[]),
        Pattern::cons("i", &[])
    ];

    for conf in confs.iter() {
        a.add(conf.clone(), (), ());
    }

    for conf in confs.iter() {
        assert!(*a.states_for_configuration(&conf).next().unwrap() == ((), ()));
        assert!(a.configurations_for_state(&()).find(|(c, _)| c == conf).is_some());
    }
}
