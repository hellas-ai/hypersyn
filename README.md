# HyperSyn: Macro Syntax for Open Hypergraphs

This library provides a macro-based syntax frontend for constructing
[open hypergraphs](https://docs.rs/open-hypergraphs/latest/open_hypergraphs/).
The main idea is to use the host language (Rust) to write "metaprograms" which
generate syntax -- open-hypergraphs.

This library builds on the
[`Var` interface](https://docs.rs/open-hypergraphs/latest/open_hypergraphs/lax/var/index.html)
of [Open Hypergraphs](https://docs.rs/open-hypergraphs/latest/open_hypergraphs/)
using some helper macros.

In short, you can write something like this:

```rust
#[def_arrow(Bit, Gate, and_arrow)]
fn and(a: var!(Bit), b: var!(Bit)) -> (var!(Bit), var!(Bit)) {
    (a.clone(), a & b)
}
```

This expands the original function by augmenting it with a `state` parameter:

```rust
fn and(
    state: Rc<RefCell<OpenHypergraph<Bit, Gate>>>,
    a: Var<Bit, Gate>,
    b: Var<Bit, Gate>,
) -> (Var<Bit, Gate>, Var<Bit, Gate>) {
    (a.clone(), a & b)
}
```

... and also defines a function `and_arrow` which constructs an `OpenHypergraph` by treating the arguments annotated with `var!` as the *sources* of your open hypergraph, and the output values as its *targets*.
Essentially, this expands to the following boilerplate:

```rust
fn and_arrow() -> OpenHypergraph<Bit, Gate> {
    let builder = Rc::new(RefCell::new(OpenHypergraph::<Bit, Gate>::empty()));
    {
        let a = Var::new(builder.clone(), Bit);
        let b = Var::new(builder.clone(), Bit);
        
        builder.borrow_mut().sources = vec![a.new_source(), b.new_source()];

        let result = and(builder.clone(), a, b);

        builder.borrow_mut().targets = {
            let (r_0, r_1) = result;
            vec![r_0.new_source(), r_1.new_source()]
        };
    }
    std::rc::Rc::try_unwrap(builder).unwrap().into_inner()
}
```

