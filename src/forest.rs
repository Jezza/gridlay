use crate::Error;
use crate::geo::{Props, Layout};
use crate::geo::Point;
use crate::geo::RectExt;
use crate::geo::Size;
use crate::geo::Rect;
use crate::geo::Number;
use crate::Template;
use crate::NodeId;
use std::collections::HashMap;

pub(crate) struct NodeData {
	pub(crate) props: Props,
//	pub(crate) layout: Option<Rect>,
	pub(crate) template: Option<Template>,
//	pub(crate) layout_cache: Option<Cache>,
	is_dirty: bool,
}

impl NodeData {
	fn new_leaf(props: Props) -> Self {
		NodeData {
			props,
//			layout: None,
			template: None,
//			layout_cache: None,
			is_dirty: true,
		}
	}

	fn new(props: Props, template: Template) -> Self {
		NodeData {
			props,
//			layout: None,
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
	pub(crate) debug: HashMap<NodeId, &'static str>,
}

impl Forest {
	pub fn with_capacity(capacity: usize) -> Self {
		Forest {
			nodes: Vec::with_capacity(capacity),
			children: Vec::with_capacity(capacity),
			parents: Vec::with_capacity(capacity),
			debug: HashMap::with_capacity(capacity),
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

	pub fn compute_layout(&mut self, node: NodeId) -> Result<Layout, Error> {
//		let point = Point::new(0 as Number, 0 as Number);
//		let size = Size::new(1 as Number, 1 as Number);
//		let rect = Rect::new(point, size);

		let (size, mut table) = self.compute(node)?;
		let width = size.width;
		let height = size.height;

		table.iter_mut()
			.for_each(|(_, rect)| *rect = rect.scale(width, height));

//		for (node_id, rect) in table.iter_mut() {
//			*rect = rect.scale(width, height);
//		}
		for (node_id, name) in self.debug.iter() {
			if table.contains_key(node_id) {
				println!("{} = {}", node_id, name);
			}
		}
		println!("Absolute => {:#?}", table);

		Ok(Layout {
			size,
			table,
		})
	}
}

impl Forest {
	pub(crate) fn compute(&mut self, root: NodeId) -> Result<(Size, HashMap<NodeId, Rect>), Error> {
		if self.children[root].is_empty() {
			let size = self.nodes[root].props.size
				.ok_or_else(|| Error("Leaf node has no defined size.".into()))?;
			return Ok((size, HashMap::new()));
		}

		let mut width = 0 as Number;
		let mut height = 0 as Number;
		let mut table = HashMap::new();

		let template = self.nodes[root].template.as_ref().unwrap();
		let (template_width, template_height) = template.size;
		for (template_rect, child_id) in template.iter()? {
			let name = self.debug[&child_id];

			println!();
			println!("{} => Template({:?} at {:?})", name, template_rect.size, template_rect.origin);

			let relative_rect = template_rect.relativise(template_width as Number, template_height as Number);
			println!("{} => RelativeTemplate({:?} at {:?})", name, relative_rect.size, relative_rect.origin);

			let computed_size = {
				let (computed_size, mut relative_table) = self.compute(child_id)?;

				if relative_table.is_empty() {
					table.insert(child_id, relative_rect);
				} else {
					for (node_id, relative) in relative_table.iter_mut() {
						relative.size = {
							Size::new(
								relative.size.width * relative_rect.size.width,
								relative.size.height * relative_rect.size.height,
							)
						};

						relative.origin = {
							Point::new(
								relative_rect.origin.x + (relative.origin.x * relative_rect.size.width),
								relative_rect.origin.y + (relative.origin.y * relative_rect.size.height),
							)
						};
					}

					table.extend(relative_table);
				}

//				template_rect.size.max(computed_size)
				computed_size
			};

			width = width.max(computed_size.width * (1 as Number / relative_rect.size.width));
			height = height.max(computed_size.height * (1 as Number / relative_rect.size.height));

			println!("{} => Result({:?} at {:?})", name, computed_size, relative_rect.origin);
		}

		println!("({:?}, {:?}) => {:#?}", width, height, table);

		Ok((Size::new(width, height), table))
	}
}