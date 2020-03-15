use crate::geo::{Rect, Number, Point, Size};
use crate::{Node, Error, GridLay, NodeId};

#[derive(Debug)]
enum StateMachine {
	Start,
	FirstRow {
		count: usize
	},
	Rest {
		width: usize,
		count: usize,
		height: usize,
	},
}

impl StateMachine {
	fn new() -> Self {
		StateMachine::Start
	}

	fn push_column(&mut self) -> Result<(), Error> {
//		println!("column = {:#?}", self);
		match self {
			StateMachine::Start => *self = StateMachine::FirstRow {
				count: 1,
			},
			StateMachine::FirstRow {
				count
			} => *count += 1,
			StateMachine::Rest {
				width,
				count,
				height,
			} => {
				if count > width {
					return Err(Error(format!("Line {} is unbalanced. [expected: {}, got: {}]", *height + 1, *width, *count)));
				}
				*count += 1;
			}
		}
		Ok(())
	}

	fn push_row(&mut self) -> Result<(), Error> {
//		println!("row = {:#?}", self);
		match self {
			StateMachine::Start => return Err(Error("Zero width templates are not supported.".into())),
			StateMachine::FirstRow {
				count
			} => {
				*self = StateMachine::Rest {
					width: *count,
					count: 0,
					height: 1,
				}
			},
			StateMachine::Rest {
				width,
				count,
				height,
			} => {
				if count != width {
					return Err(Error(format!("Line {} is unbalanced. [expected: {}, got: {}]", *height + 1, *width, *count)));
				}
				*count = 0;
				*height += 1;
			}
		}
		Ok(())
	}

	fn size(&self) -> (usize, usize) {
//		println!("end = {:#?}", self);
		match self {
			StateMachine::Start => unreachable!(),
			StateMachine::FirstRow {
				count
			} => {
				(*count, 1)
			},
			StateMachine::Rest {
				width,
				height,
				..
			} => {
				(*width, *height)
			}
		}
	}
}

pub struct Lines<'a> {
	grid: &'a mut GridLay,
	data: Vec<Vec<NodeId>>,
	machine: StateMachine,
}

impl Lines<'_> {

	pub(crate) fn new(grid: &mut GridLay) -> Lines<'_> {
		Lines {
			grid,
			data: vec![],
			machine: StateMachine::new()
		}
	}

	pub fn add(&mut self, node: Node) -> Result<(), Error> {
		let node = self.grid.find_node(node)?;
		if let Some(line) = self.data.last_mut() {
			line.push(node);
		} else {
			self.data.push(vec![node]);
		}
		self.machine.push_column()?;
		Ok(())
	}

	pub fn end(&mut self) -> Result<(), Error> {
		self.data.push(vec![]);
		self.machine.push_row()
	}

	pub(crate) fn into_template(self) -> Template {
		let Lines {
			mut data,
			machine,
			..
		} = self;
		if let Some(line) = data.last() {
			if line.is_empty() {
				data.pop();
			}
		}

		let data: Vec<NodeId> = data.into_iter()
			.flat_map(|d| d.into_iter())
			.collect();

		let (width, height) = machine.size();

		Template {
			data,
			size: (width, height)
		}
	}
}

pub struct Template {
	pub(crate) data: Vec<NodeId>,
	pub(crate) size: (usize, usize),
}

impl Template {
	pub(crate) fn ids(&self) -> Vec<NodeId> {
		let mut children: Vec<NodeId> = self.data.clone();

		children.sort();
		children.dedup();

		children
	}

	pub(crate) fn iter(&self) -> Result<impl Iterator<Item = (NodeId, Rect)>, Error> {

		let (width, height) = self.size;

		let len = width * height;

		let mut index = 0;
		let mut points = vec![];
		let mut bit_mask = vec![false; len];
		let mut seen = vec![];

		macro_rules! point {
			($index:ident) => {{
				let x = $index % width;
				let y = $index / width;
				(x, y)
			}};
		}

		while index < len {
			if bit_mask[index] {
				index += 1;
				continue;
			}
			let node = self.data.get(index)
				.unwrap();

			if seen.contains(node) {
				return Err(Error(format!("Invalid character at {:?}. [Forms invalid shape: {:?}]", point!(index), node)))
			}

			let rect_width = {
				let mut rect_width = width;
				for offset in 1..width {
					let right = index + offset;

					if right >= len || self.data.get(right).unwrap() != node {
						rect_width = offset;
						break;
					}
				};
				rect_width
			};

			let rect_height = {
				let mut rect_height = height;
				for offset in 1..height {
					let below = index + (offset * width);

					if below >= len || self.data.get(below).unwrap() != node {
						rect_height = offset;
						break;
					}
				}
				rect_height
			};

			for y_offset in 0..rect_height {
				for x_offset in 0..rect_width {
					let bit_index = index + x_offset + (y_offset * width);
					let at = self.data.get(bit_index).unwrap();
					if at != node {
						return Err(Error(format!("Invalid character at {:?}. [expected: {:?}, found: {:?}]", point!(bit_index), node, at)))
					}
					bit_mask[bit_index] = true;
				}
			}

			let (x, y) = point!(index);

			let point = Point::new(x as Number, y as Number);
			let size = Size::new(rect_width as Number, rect_height as Number);
			let layout = Rect::new(point, size);

			seen.push(*node);

			points.push((*node, layout));
		}

		Ok(points.into_iter())
	}
}