use std::collections::HashMap;

use crate::Node;
use crate::geo::{Layout, Number, Rect};

pub fn print_layout<S: ::std::hash::BuildHasher>(layout: &Layout, names: &HashMap<Node, &'static str, S>) -> String {
	let Layout {
		size,
		table,
	} = layout;

	let scale = 1usize;

	let mut printer = GridPrinter::new(size.width as usize * scale, size.height as usize * scale);

	for (node, rect) in table {
		let Rect {
			size,
			origin,
		} = rect;

		let name = names.get(node).unwrap_or(&"<unknown>");

		for x in 0..((size.width * scale as Number) as usize) {
			for y in 0..((size.height * scale as Number) as usize) {
				printer.set(origin.x as usize * scale + x, origin.y as usize * scale + y, name);
			}
		}
	}

	printer.display()
}

struct GridPrinter {
	width: usize,
	data: Vec<&'static str>,
	max: usize,
}

const DEFAULT: &str = "<unset>";

impl GridPrinter {
	fn new(width: usize, height: usize) -> Self {
		GridPrinter {
			width,
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

