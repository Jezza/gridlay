use gridlay::geo::{Props, Rect};
use gridlay::geo::Layout;
use gridlay::{GridLay, NodeId};
use gridlay::lines;
use std::collections::HashMap;

pub fn main() {
	let mut grid = GridLay::new();

	let a = grid.new_leaf(Props::sized(1.0, 1.0));
	let b = grid.new_leaf(Props::sized(2.0, 2.0));
	let c = grid.new_leaf(Props::sized(1.0, 3.0));
	grid.debug(a, "a").unwrap();
	grid.debug(b, "b").unwrap();
	grid.debug(c, "c").unwrap();

	let parent = grid.new_node(lines! {
			a b;
			c c;
		}).unwrap();
	grid.debug(parent, "parent").unwrap();

	let d = grid.new_leaf(Props::sized(1.0, 3.0));
	grid.debug(d, "d").unwrap();

	let root = grid.new_node(lines! {
			d parent parent parent parent parent parent parent parent parent ;
		}).unwrap();

	let layout = grid.compute_layout(root).unwrap();

	println!("{}", print_layout(&layout, grid.names()));
}

fn print_layout(layout: &Layout, names: &HashMap<NodeId, &'static str>) -> String {
	let Layout {
		size,
		table,
	} = layout;

	let mut printer = GridPrinter::new(size.width as usize, size.height as usize);

	for (node_id, rect) in table {
		let Rect {
			size,
			origin,
		} = rect;

		let name = names[node_id];

		for x in 0..(size.width as usize) {
			for y in 0..(size.height as usize) {
				printer.set(origin.x as usize + x, origin.y as usize + y, name);
			}
		}
	}

	printer.display()
}

struct GridPrinter {
	width: usize,
	height: usize,
	data: Vec<&'static str>,
	max: usize,
}

const DEFAULT: &str = "<unset>";

impl GridPrinter {
	fn new(width: usize, height: usize) -> Self {
		GridPrinter {
			width,
			height,
			data: vec![DEFAULT; width * height],
			max: DEFAULT.len(),
		}
	}

	fn set(&mut self, x: usize, y: usize, name: &'static str) {
		self.data[x + (y * self.width)] = name;
		self.max = self.max.max(name.len());
	}

	fn display(&self) -> String {
		let mut buf = String::new();

		let max = self.max;

		for (i, name) in self.data.iter().enumerate() {

			let mark = buf.len();

			buf += name;
			buf += " ";

			let diff = buf.len() - mark;
			if diff < max {
				for _ in 0..max - diff {
					buf += " ";
				}
			}

			if (i + 1) % self.width == 0 {
				buf += "\n";
			}
		}

		buf
	}
}

