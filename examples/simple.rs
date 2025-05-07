//! A simple example of boolean circuits.
//! We begin with some boilerplate code setting up the theory.
use open_hypergraphs::lax::var;

/// The category of boolean circuits has a single generating object, a boolean type.
#[derive(PartialEq, Clone, Debug)]
pub struct Bit;

// The generating operations are logic gates
#[derive(PartialEq, Clone, Debug)]
pub enum Gate {
    Copy,
    And,
}

// Implement trait helpers so we can use operators with `Var`s.
impl var::HasVar for Gate {
    fn var() -> Gate {
        Gate::Copy
    }
}

impl var::HasBitAnd<Bit, Gate> for Gate {
    fn bitand(_: Bit, _: Bit) -> (Bit, Gate) {
        (Bit, Gate::And)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main example code

// Import the macro
use hypersyn::def_arrow;

/// A circuit with two inputs: a and b.
/// Returns the first input, and the AND of both.
///
/// def_arrow defines an extra function and_arrow which will construct the open hypergraph
/// whose interfaces are the inputs and outputs to this function.
#[def_arrow(Bit, Gate, and_arrow)]
fn and(a: var!(Bit), b: var!(Bit)) -> (var!(Bit), var!(Bit)) {
    (a.clone(), a & b)
}

fn main() {
    println!("{:?}", and_arrow());
}
