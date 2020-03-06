use crate::Error;
use crate::geo::{Props, Point, OrElse};
use crate::geo::Size;
use crate::geo::Unit;
use crate::geo::Rect;
use crate::geo::Number;
use crate::Template;
use crate::NodeId;

pub(crate) struct NodeData {
	pub(crate) props: Props,
	pub(crate) layout: Rect,
	pub(crate) template: Option<Template>,
//	pub(crate) layout_cache: Option<Cache>,
	is_dirty: bool,
}

impl NodeData {
	fn new_leaf(props: Props) -> Self {
		NodeData {
			props,
			layout: Rect::new(Point::new(Unit::Undefined, Unit::Undefined), Size::new(Unit::Undefined, Unit::Undefined)),
			template: None,
//			layout_cache: None,
			is_dirty: true,
		}
	}

	fn new(props: Props, template: Template) -> Self {
		NodeData {
			props,
			layout: Rect::new(Point::new(Unit::Undefined, Unit::Undefined), Size::new(Unit::Undefined, Unit::Undefined)),
			template: Some(template),
//			layout_cache: None,
			is_dirty: true,
		}
	}
}

pub(crate) struct Cache;

pub(crate) struct Forest {
	pub(crate) nodes: Vec<NodeData>,
	pub(crate) children: Vec<Vec<NodeId>>,
	pub(crate) parents: Vec<Vec<NodeId>>,
}

impl Forest {
	pub fn with_capacity(capacity: usize) -> Self {
		Forest {
			nodes: Vec::with_capacity(capacity),
			children: Vec::with_capacity(capacity),
			parents: Vec::with_capacity(capacity),
		}
	}

	pub fn new_leaf(&mut self, props: Props) -> NodeId {
		let id = self.nodes.len();
		self.nodes.push(NodeData::new_leaf(props));
		self.children.push(Vec::with_capacity(0));
		self.parents.push(Vec::with_capacity(1));
		id
	}

	pub fn new_node(&mut self, props: Props, template: Template) -> NodeId {
		let id = self.nodes.len();

		let children = template.ids();

		for child in &children {
			self.parents[*child].push(id);
		}
		self.nodes.push(NodeData::new(props, template));
		self.children.push(children);
		self.parents.push(Vec::with_capacity(1));
		id
	}

	pub fn add_child(&mut self, node: NodeId, child: NodeId) {
		self.parents[child].push(node);
		self.children[node].push(child);
		self.mark_dirty(node)
	}

	pub fn clear(&mut self) {
		self.nodes.clear();
		self.children.clear();
		self.parents.clear();
	}

	/// Removes a node and swaps with the last node.
	pub fn swap_remove(&mut self, node: NodeId) -> Option<NodeId> {
		self.nodes.swap_remove(node);

		// Now the last element is swapped in at index `node`.
		if self.nodes.is_empty() {
			self.children.clear();
			self.parents.clear();
			return None;
		}

		// Remove old node as parent from all its chilren.
		for child in &self.children[node] {
			let parents_child = &mut self.parents[*child];
			let mut pos = 0;
			while pos < parents_child.len() {
				if parents_child[pos] == node {
					parents_child.swap_remove(pos);
				} else {
					pos += 1;
				}
			}
		}

		// Remove old node as child from all its parents.
		for parent in &self.parents[node] {
			let childrens_parent = &mut self.children[*parent];
			let mut pos = 0;
			while pos < childrens_parent.len() {
				if childrens_parent[pos] == node {
					childrens_parent.swap_remove(pos);
				} else {
					pos += 1;
				}
			}
		}

		let last = self.nodes.len();

		if last != node {
			// Update ids for every child of the swapped in node.
			for child in &self.children[last] {
				for parent in &mut self.parents[*child] {
					if *parent == last {
						*parent = node;
					}
				}
			}

			// Update ids for every parent of the swapped in node.
			for parent in &self.parents[last] {
				for child in &mut self.children[*parent] {
					if *child == last {
						*child = node;
					}
				}
			}

			self.children.swap_remove(node);
			self.parents.swap_remove(node);

			Some(last)
		} else {
			self.children.swap_remove(node);
			self.parents.swap_remove(node);
			None
		}
	}

	pub unsafe fn remove_child(&mut self, node: NodeId, child: NodeId) -> NodeId {
		let index = self.children[node].iter().position(|n| *n == child).unwrap();
		self.remove_child_at_index(node, index)
	}

	pub fn remove_child_at_index(&mut self, node: NodeId, index: usize) -> NodeId {
		let child = self.children[node].remove(index);
		self.parents[child].retain(|p| *p != node);
		self.mark_dirty(node);
		child
	}

	pub fn mark_dirty(&mut self, node: NodeId) {
		fn mark_dirty_impl(nodes: &mut Vec<NodeData>, parents: &[Vec<NodeId>], node_id: NodeId) {
			let node = &mut nodes[node_id];
//			node.layout_cache = None;
			node.is_dirty = true;

			for parent in &parents[node_id] {
				mark_dirty_impl(nodes, parents, *parent);
			}
		}

		mark_dirty_impl(&mut self.nodes, &self.parents, node);
	}

	pub fn compute_layout(&mut self, node: NodeId) -> Result<(), Error> {
		let point = Point::new(Unit::Defined(0.0), Unit::Defined(0.0));
		let size = Size::new(Unit::Defined(1.0), Unit::Defined(1.0));
		let rect = Rect::new(point, size);
		self.compute(node, rect).map(|_| ())
	}
}

impl Forest {
	pub(crate) fn compute(&mut self, root: NodeId, rect: Rect) -> Result<Size, Error> {
		if self.children[root].is_empty() {
			let size = self.nodes[root].props.size;

			if size.height == Unit::Undefined {
				return Err(Error("Leaf node has no defined height.".into()));
			}
			if size.width == Unit::Undefined {
				return Err(Error("Leaf node has no defined width.".into()));
			}

			self.nodes[root].layout = Rect::new(rect.origin, size);

			return Ok(size);
		}

		let mut width = 0 as Number;
		let mut height = 0 as Number;

		let template = self.nodes[root].template.as_ref().unwrap();
		let (template_width, template_height) = template.size;
		for (rect, child_id) in template.iter()? {
			println!("Child: {:?}", rect);
//			let x = x + rect.location.x.or_else(0.0);
//			let y = y + rect.location.y.or_else(0.0);
//
//			let node_width = rect.size.width.or_else(0.0).max(self.nodes[node_id].props.size.width.or_else(0.0));
//			let node_height = rect.size.height.or_else(0.0).max(self.nodes[node_id].props.size.height.or_else(0.0));
//
//			let size = Size::new(Unit::Defined(node_width), Unit::Defined(node_height));
//
//			self.compute(node_id, size, x, y)?;
//			let layout = self.nodes[node_id].layout;
//			println!("Result: {:?}", layout);

//			width += node_width;
//			height += node_height;
		}

//		self.nodes[root].layout = {
//			let mut layout = Layout::new();
//			layout.size = Size::new(Unit::Defined(width), Unit::Defined(height));
//			layout.location = Point::new(Unit::Defined(x), Unit::Defined(y));
//			println!("Nested: {:?}", layout);
//			layout
//		};


		Ok(Size::new(Unit::Undefined, Unit::Undefined))
	}
}