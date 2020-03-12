use gridlay::geo::Props;
use gridlay::GridLay;
use gridlay::lines;

pub fn main() {
	let mut grid = GridLay::new();

	let a = grid.new_leaf(Props::sized(2.0, 1.0));
	let b = grid.new_leaf(Props::sized(1.0, 3.0));
	let c = grid.new_leaf(Props::sized(1.0, 2.0));
	let d = grid.new_leaf(Props::sized(1.0, 1.0));

	grid.debug(a, "a").unwrap();
	grid.debug(b, "b").unwrap();
	grid.debug(c, "c").unwrap();
	grid.debug(d, "d").unwrap();

	let parent = grid.new_node(lines! {
			a a a;
			b c d;
		}).unwrap();
	grid.debug(parent, "parent");

	let e = grid.new_leaf(Props::sized(1.0, 2.0));
	grid.debug(e, "e");

	let root = grid.new_node(lines! {
			e parent;
		}).unwrap();
	grid.debug(root, "root");

	grid.compute_layout(root).unwrap();

//	d a a;
//	d a a;
//	d b c;
//	d b c;
//	d b c;
//	d b c;

//	a a;
//	b c;
//	b c;

//	d a a
//	d b c
//	d b c

//	let layout = grid.layout(parent).unwrap();
//	println!("{:?}", layout);
//
//	let mut buffer = String::new();
//
//	for node in grid.children(parent).unwrap() {
//	}
}