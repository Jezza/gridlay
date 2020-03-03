use gridlay::geo::Props;
use gridlay::GridLay;
use gridlay::lines;

pub fn main() {
	let mut grid = GridLay::new();

	let a = grid.new_leaf(Props::sized(2.0, 1.0));
	let b = grid.new_leaf(Props::sized(1.0, 2.0));
	let c = grid.new_leaf(Props::sized(1.0, 2.0));

	let parent = grid.new_node(lines! {
			a a;
			b c;
			b c;
		}).unwrap();


	let d = grid.new_leaf(Props::sized(1.0, 2.0));

	let parent = grid.new_node(lines! {
			d parent;
			d parent;
			d parent;
			d parent;
			d parent;
			d parent;
		}).unwrap();

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

	grid.compute_layout(parent).unwrap();

	let layout = grid.layout(parent).unwrap();
	println!("{:?}", layout);

	let mut buffer = String::new();

	for node in grid.children(parent).unwrap() {

		

	}
}