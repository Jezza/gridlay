#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::forest::Forest;
pub use crate::geo::{Props, Layout};
pub use crate::template::Lines;
use crate::id::NodeId;

mod id;
mod forest;
mod template;
pub mod printer;
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

impl Default for GridLay {
	fn default() -> Self {
		Self::new()
	}
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

	pub fn clear(&mut self) {
		self.nodes_to_ids.clear();
		self.ids_to_nodes.clear();
		self.forest.clear();
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
		let id = self.forest.new_leaf(props);

		let node = self.allocate_node();
		self.add_node(node, id);
		node
	}

	pub fn new_node<F>(&mut self, layout: F) -> Result<Node, Error> where F: Fn(&mut Lines<'_>) -> Result<(), Error> {
		let template = {
			let mut lines = Lines::new(self);
			layout(&mut lines)?;
			lines.into_template()
		};

		let id = self.forest.new_node(template);

		let node = self.allocate_node();
		self.add_node(node, id);
		Ok(node)
	}

//	pub fn children(&self, node: Node) -> Result<Vec<Node>, Error> {
//		let id = self.find_node(node)?;
//		Ok(self.forest.children[id].iter().map(|child| self.ids_to_nodes[child]).collect())
//	}

//	pub fn child_count(&self, node: Node) -> Result<usize, Error> {
//		let id = self.find_node(node)?;
//		Ok(self.forest.children[id].len())
//	}

//	pub fn layout(&self, node: Node) -> Result<Option<&Rect>, Error> {
//		let id = self.find_node(node)?;
//		Ok(self.forest.nodes[id].layout.as_ref())
//	}

	pub fn compute_layout(&mut self, node: Node) -> Result<Layout, Error> {
		let id = self.find_node(node)?;

		let GridLay {
			forest,
			ids_to_nodes,
			..
		} = self;

		forest.compute_layout(id, ids_to_nodes)
	}
}


impl Drop for GridLay {
	fn drop(&mut self) {
		INSTANCE_ALLOCATOR.lock()
			.unwrap()
			.free(&[self.id]);
	}
}
