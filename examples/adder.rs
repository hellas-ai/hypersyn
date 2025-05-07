//! An example of building circuits using the `def_arrow` macro.
//!
//! We start by defining the theory of boolean circuits, then
//! define a full adder, and print its dot source code.
use open_hypergraphs::lax::var;
use std::hash::Hash;

// There is a single generating object in the category: the bit.
#[derive(PartialEq, Clone, Debug, Hash)]
pub struct Bit;

// The generating operations are logic gates
#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Gate {
    Not,
    Xor,
    Zero, // 0 → 1
    Or,
    And,
    One,
    Copy, // explicit copying of values
}

impl var::HasVar for Gate {
    fn var() -> Gate {
        Gate::Copy
    }
}

impl var::HasBitXor<Bit, Gate> for Gate {
    fn bitxor(_: Bit, _: Bit) -> (Bit, Gate) {
        (Bit, Gate::Xor)
    }
}

impl var::HasBitAnd<Bit, Gate> for Gate {
    fn bitand(_: Bit, _: Bit) -> (Bit, Gate) {
        (Bit, Gate::And)
    }
}

impl var::HasBitOr<Bit, Gate> for Gate {
    fn bitor(_: Bit, _: Bit) -> (Bit, Gate) {
        (Bit, Gate::Or)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main example code

use hypersyn::def_arrow;

#[def_arrow(Bit, Gate, full_adder_arrow)]
fn full_adder(a: var!(Bit), b: var!(Bit), cin: var!(Bit)) -> (var!(Bit), var!(Bit)) {
    // we reuse this computation twice, so bind it here.
    // This implicitly creats a Copy edge
    let a_xor_b = a.clone() ^ b.clone();

    let sum = a_xor_b.clone() ^ cin.clone();
    let cout = (a & b) | (cin & a_xor_b.clone());

    (sum, cout)
}

////////////////////////////////////////
// Lay out the adder circuit using dot

use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec,
    printer::PrinterContext,
};
use open_hypergraphs_dot::{dark_theme, generate_dot};

fn main() -> std::io::Result<()> {
    let open_hypergraph = full_adder_arrow();
    let dot_graph = generate_dot(&open_hypergraph, &dark_theme());

    let png_bytes = exec(
        dot_graph,
        &mut PrinterContext::default(),
        vec![CommandArg::Format(Format::Png)],
    )?;
    std::fs::write("examples/adder.png", png_bytes)?;
    Ok(())
}
