#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::forest::Forest;
use crate::geo::{Layout, Rect};
use crate::geo::Point;
use crate::geo::Props;
use crate::geo::Size;
use crate::geo::Unit;

mod id;
mod forest;
pub mod geo;

#[macro_export]
macro_rules! lines {
    (
		$(
			$(
				$var:ident
			)*
			;
		)*
    ) => {{
    	|lines| {
			$(
				$(
					lines.add($var)?;
				)*
				lines.end()?;
			)*
			Ok(())
    	}
    }};
}

#[derive(Debug)]
pub struct Error(String);

lazy_static! {
    /// Global stretch instance id allocator.
    static ref INSTANCE_ALLOCATOR: Mutex<id::Allocator> = Mutex::new(id::Allocator::new());
}

pub type NodeId = usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Node {
	instance: id::Id,
	local: id::Id,
}

pub struct GridLay {
	id: id::Id,
	allocator: id::Allocator,
	nodes_to_ids: HashMap<Node, NodeId>,
	ids_to_nodes: HashMap<NodeId, Node>,
	forest: forest::Forest,
}

impl GridLay {
	pub fn new() -> Self {
		Self::with_capacity(16)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		GridLay {
			id: INSTANCE_ALLOCATOR.lock().unwrap().allocate(),
			allocator: id::Allocator::new(),
			nodes_to_ids: HashMap::with_capacity(capacity),
			ids_to_nodes: HashMap::with_capacity(capacity),
			forest: Forest::with_capacity(capacity),
		}
	}

	fn allocate_node(&mut self) -> Node {
		let local = self.allocator.allocate();
		Node {
			instance: self.id,
			local,
		}
	}

	fn add_node(&mut self, node: Node, id: NodeId) {
		self.nodes_to_ids.insert(node, id);
		self.ids_to_nodes.insert(id, node);
	}

	fn find_node(&self, node: Node) -> Result<NodeId, Error> {
		match self.nodes_to_ids.get(&node) {
			Some(id) => Ok(*id),
			None => Err(Error("Unable to find node.".into())),
		}
	}

	pub fn new_leaf(&mut self, props: Props) -> Node {
		let node = self.allocate_node();
		let id = self.forest.new_leaf(props);
		self.add_node(node, id);
		node
	}

	pub fn new_node<F>(&mut self, layout: F) -> Result<Node, Error> where F: Fn(&mut Lines<'_>) -> Result<(), Error> {
		let props = Props::undefined();
		let template = {
			let mut lines = Lines::new(self);
			layout(&mut lines)?;
			lines.into_template()
		};

		let id = self.forest.new_node(props, template);

		let node = self.allocate_node();
		self.add_node(node, id);
		Ok(node)
	}

	pub fn children(&self, node: Node) -> Result<Vec<Node>, Error> {
		let id = self.find_node(node)?;
		Ok(self.forest.children[id].iter().map(|child| self.ids_to_nodes[child]).collect())
	}

	pub fn child_count(&self, node: Node) -> Result<usize, Error> {
		let id = self.find_node(node)?;
		Ok(self.forest.children[id].len())
	}

	pub fn layout(&self, node: Node) -> Result<&Layout, Error> {
		let id = self.find_node(node)?;
		Ok(&self.forest.nodes[id].layout)
	}

	pub fn compute_layout(&mut self, node: Node) -> Result<(), Error> {
		let id = self.find_node(node)?;
		self.forest.compute_layout(id)
	}
}


impl Drop for GridLay {
	fn drop(&mut self) {
		INSTANCE_ALLOCATOR.lock()
			.unwrap()
			.free(&[self.id]);
	}
}

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

	fn new(grid: &mut GridLay) -> Lines<'_> {
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

	fn into_template(self) -> Template {
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
	data: Vec<NodeId>,
	size: (usize, usize),
}

impl Template {
	fn ids(&self) -> Vec<NodeId> {
		let mut children: Vec<NodeId> = self.data.clone();

		children.sort();
		children.dedup();

		children
	}

	fn iter(&self) -> Result<impl Iterator<Item = (Layout, NodeId)>, Error> {

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

			let point = Point::new(Unit::Defined(x as f32), Unit::Defined(y as f32));
			let size = Size::new(Unit::Defined(rect_width as f32), Unit::Defined(rect_height as f32));
			let layout = Rect::new(point, size);

			seen.push(*node);

			points.push((layout, *node));
		}

		Ok(points.into_iter())
	}
}