# GridLay

A simple Rust grid layout engine.

```rust
use gridlay::{GridLay, lines, Props};

pub fn main() {
    let mut grid = GridLay::new();

    let a = grid.new_leaf(Props::sized(1.0, 1.0));
    let b = grid.new_leaf(Props::sized(2.0, 2.0));
    let c = grid.new_leaf(Props::sized(1.0, 3.0));

    let parent = grid.new_node(lines! {
            a b;
            c c;
        }).unwrap();

    let d = grid.new_leaf(Props::sized(1.0, 3.0));

    let root = grid.new_node(lines! {
            d parent;
        }).unwrap();

    let layout = grid.compute_layout(root).unwrap();

    // `layout` contains each node's layout data (size and location),
    // as well, as the total size of the layout.
}
```


I designed it to be simple, because I only need a simple layout engine.  
I'm using this to dogfeed my `kog` project.

If you have any issues, questions, or suggestions, feel free to open an issue.
