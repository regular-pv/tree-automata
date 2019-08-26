#[macro_export]
macro_rules! automaton {
    ( $( $token:tt )* ) => {
        {
            let mut automaton = Automaton::new();
            automaton_items!(automaton $( $token )*);
            automaton
        }
    }
}

#[macro_export]
macro_rules! automaton_items {
    ( $aut:ident $f:tt ( $( $sub:expr ),* ) -> $q:tt, $( $token:tt )* ) => {
        let mut subs = Vec::new();
        $(
            subs.push($sub.clone());
        )*
        $aut.add(Configuration($f.clone(), subs), NoLabel, $q);
        automaton_items!($aut $( $token )*)
    };
    ( $aut:ident $f:tt -> $q:tt, $( $token:tt )* ) => {
        $aut.add(Configuration($f.clone(), Vec::new()), NoLabel, $q);
        automaton_items!($aut $( $token )*)
    };
    ( $aut:ident finals $($q:tt)* ) => {
        $(
            $aut.set_final($q);
        )*
    }
}
