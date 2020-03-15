use std::collections::HashMap;

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

	let mut names = HashMap::new();

	names.insert(a, "a");
	names.insert(b, "b");
	names.insert(c, "c");
	names.insert(d, "d");

	println!("{}", gridlay::printer::print_layout(&layout, &names));
}
