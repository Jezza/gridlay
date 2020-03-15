use crate::{Error, Node};
use crate::geo::{Props, Layout};
use crate::geo::Point;
use crate::geo::RectExt;
use crate::geo::Size;
use crate::geo::Rect;
use crate::geo::Number;
use crate::template::Template;
use crate::NodeId;
use std::collections::HashMap;

pub(crate) enum Data {
	Leaf {
		props: Props,
	},
	Parent {
		template: Template,
	}
}

impl Data {
	fn new_leaf(props: Props) -> Self {
		Data::Leaf {
			props,
		}
	}

	fn new_parent(template: Template) -> Self {
		Data::Parent {
			template,
		}
	}
}

pub(crate) struct Forest {
	pub(crate) nodes: Vec<Data>,
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
		self.nodes.push(Data::new_leaf(props));
		self.children.push(Vec::with_capacity(0));
		self.parents.push(Vec::with_capacity(1));
		id
	}

	pub fn new_node(&mut self, template: Template) -> NodeId {
		let id = self.nodes.len();

		let children = template.ids();

		for child in &children {
			self.parents[*child].push(id);
		}
		self.nodes.push(Data::new_parent(template));
		self.children.push(children);
		self.parents.push(Vec::with_capacity(1));
		id
	}

	pub fn clear(&mut self) {
		self.nodes.clear();
		self.children.clear();
		self.parents.clear();
	}

	pub fn compute_layout(&mut self, node: NodeId, ids_to_nodes : &HashMap<NodeId, Node>) -> Result<Layout, Error> {
		let (size, table) = self.compute(node)?;
		let width = size.width;
		let height = size.height;

//		println!("Relative => {:#?}", table);
		let table = table.into_iter()
			.map(|(node_id, rect)| {
				let node_id = *ids_to_nodes.get(&node_id)
					.expect("Unable to locate original node.");
				(node_id, rect.scale(width, height))
			})
			.collect();
//		println!("Absolute => {:#?}", table);

		Ok(Layout {
			size,
			table,
		})
	}
}

impl Forest {
	pub(crate) fn compute(&mut self, root: NodeId) -> Result<(Size, HashMap<NodeId, Rect>), Error> {

		let node = self.nodes.get(root)
			.ok_or_else(|| Error("Node does not exist.".into()))?;

		if self.children[root].is_empty() {
			return if let Data::Leaf {
				props
			} = node {
				Ok((props.size, HashMap::new()))
			} else {
				Err(Error("Node has no children, but is not a leaf node.".into()))
			};
		}

		let template = if let Data::Parent {
			template
		} = node {
			template
		} else {
			return Err(Error("Node has children, but is not a parent node.".into()))
		};


		let mut width = 0 as Number;
		let mut height = 0 as Number;
		let mut table = HashMap::new();

		let (template_width, template_height) = template.size;
		for (child_id, template_rect) in template.iter()? {
//			println!();
//			println!("{} => Template({:?} at {:?})", name, template_rect.size, template_rect.origin);

			let relative_rect = template_rect.relativise(template_width as Number, template_height as Number);
//			println!("{} => RelativeTemplate({:?} at {:?})", name, relative_rect.size, relative_rect.origin);

			let computed_size = {
				let (computed_size, mut relative_table) = self.compute(child_id)?;

				if relative_table.is_empty() {
					table.insert(child_id, relative_rect);
				} else {
					for (_, relative) in relative_table.iter_mut() {
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

				template_rect.size.max(computed_size)
			};

			width = width.max(computed_size.width * (1.0 / relative_rect.size.width));
			height = height.max(computed_size.height * (1.0 / relative_rect.size.height));
//			println!("{} => Result({:?} at {:?})", name, computed_size, relative_rect.origin);
		}
//		println!("({:?}, {:?}) => {:#?}", width, height, table);

		Ok((Size::new(width, height), table))
	}
}