pub type Size = euclid::default::Size2D<Unit>;
pub type Point = euclid::default::Point2D<Unit>;


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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
	Defined(Number),
	Undefined,
}

impl Default for Unit {
	fn default() -> Self {
		Unit::Undefined
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
	pub location: Point,
	pub size: Size,
}

impl Layout {
	pub fn new() -> Self {
		Layout {
			location: Point::new(Unit::Undefined, Unit::Undefined),
			size: Size::new(Unit::Undefined, Unit::Undefined),
		}
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
