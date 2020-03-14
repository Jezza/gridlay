use std::fmt::{Debug, Error, Formatter};
use std::collections::HashMap;
use crate::NodeId;

pub type Size = euclid::default::Size2D<Number>;
pub type Point = euclid::default::Point2D<Number>;
pub type Rect = euclid::default::Rect<Number>;

pub type Number = f32;

pub trait RectExt {
	fn relativise(&self, x: Number, y: Number) -> Self;
}

impl RectExt for Rect {
	fn relativise(&self, x: Number, y: Number) -> Rect {
		Rect::new(
			Point::new(self.origin.x / x, self.origin.y / y),
			Size::new(self.size.width / x, self.size.height / y),
		)
	}
}

#[derive(Clone, Debug)]
pub struct Layout {
	pub size: Size,
	pub table: HashMap<NodeId, Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Props {
	pub size: Option<Size>,
}

impl Props {
	pub fn sized(width: Number, height: Number) -> Props {
		Props {
			size: Some(Size::new(width, height)),
		}
	}

	pub fn undefined() -> Props {
		Props {
			size: None,
		}
	}
}
