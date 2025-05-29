//! An example of building circuits using the `def_arrow` macro.
//!
//! We start by defining the theory of boolean circuits, then
//! define a full adder, and print its dot source code.
use open_hypergraphs::lax::var;
use std::hash::Hash;

// There is a single generating object in the category: the bit.
#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Object {
    Int,
    Float,
}

// The generating operations are logic gates
#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Operation {
    Add,  // n → 1 add
    Mul,  // n → 1 product
    Copy, // 1 → n copy
}

impl var::HasVar for Operation {
    fn var() -> Operation {
        Operation::Copy
    }
}

impl var::HasAdd<Object, Operation> for Operation {
    fn add(lhs_type: Object, _: Object) -> (Object, Operation) {
        (lhs_type, Operation::Mul)
    }
}

impl var::HasMul<Object, Operation> for Operation {
    fn mul(lhs_type: Object, _: Object) -> (Object, Operation) {
        (lhs_type, Operation::Mul)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main example code

use hypersyn::def_arrow;

/// Build a term representing x^(2^n)
#[def_arrow(Object, Operation, exp_2n_arrow)]
fn exp_2n(n: usize, x: var!(Object::Int)) -> var!(Object::Int) {
    let mut x = x;
    for _ in 0..n {
        x = x.clone() * x
    }
    x
}

////////////////////////////////////////
// Lay out the adder circuit using dot

use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec,
    printer::PrinterContext,
};
use open_hypergraphs_dot::generate_dot;

fn main() -> std::io::Result<()> {
    let open_hypergraph = exp_2n_arrow(3);
    let dot_graph = generate_dot(&open_hypergraph);

    let png_bytes = exec(
        dot_graph,
        &mut PrinterContext::default(),
        vec![CommandArg::Format(Format::Png)],
    )?;
    std::fs::write("images/exponentiate.png", png_bytes)?;
    Ok(())
}
