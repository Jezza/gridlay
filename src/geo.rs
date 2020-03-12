use std::fmt::{Debug, Formatter, Error};

pub type Size = euclid::default::Size2D<Unit>;
pub type Point = euclid::default::Point2D<Unit>;
pub type Rect = euclid::default::Rect<Unit>;
pub type Layout = Rect;

pub trait OrElse<T> {
	fn or_else(&self, default: T) -> T;
}

impl OrElse<f32> for Unit {
	fn or_else(&self, default: f32) -> f32 {
		match self {
			Unit::Defined(value) => *value,
			_ => default,
		}
	}
}

pub type Number = f32;

#[derive(Clone, Copy, PartialEq)]
pub enum Unit {
	Defined(Number),
	Undefined,
}

impl Debug for Unit {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Unit::Undefined => f.write_str("N/A"),
			Unit::Defined(number) => f.write_str(&format!("{}", number))
		}
	}
}

impl Default for Unit {
	fn default() -> Self {
		Unit::Undefined
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Props {
	pub location: Point,
	pub size: Size,
}

impl Props {
	pub fn sized(width: Number, height: Number) -> Props {
		Props {
			location: Point::new(Unit::Undefined, Unit::Undefined),
			size: Size::new(Unit::Defined(width), Unit::Defined(height)),
		}
	}

	pub fn undefined() -> Props {
		Props {
			location: Point::new(Unit::Undefined, Unit::Undefined),
			size: Size::new(Unit::Undefined, Unit::Undefined),
		}
	}
}
