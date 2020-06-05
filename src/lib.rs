use druid::{
    kurbo::{Affine, BezPath, Circle, PathEl, Point, Rect, Shape, Size},
    widget::prelude::*,
    Color, Data,
};

/// A widget that displays a material icon. Use constraints to set the preferred size.
///
/// # Examples
///
/// ```
/// # use druid::{Widget, Data, Color, WidgetExt};
/// # #[derive(Data, Clone)]
/// # struct MyData;
/// use druid_material_icons as icons;
/// fn build_ui() -> impl Widget<MyData> {
///     icons::ADD.new(Color::BLACK).fix_width(12.0).center()
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Icon {
    shapes: IconShapes,
    color: Color,
}

impl Icon {
    #[inline]
    fn new(shapes: IconShapes, color: Color) -> Self {
        Self { shapes, color }
    }
}

impl<T: Data> Widget<T> for Icon {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {
        // no events
    }
    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {
        // no lifecycle
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {
        // no update
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, _env: &Env) -> Size {
        max_with_aspect(bc, aspect_ratiof(self.shapes.size))
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, _env: &Env) {
        let Size { width, height } = ctx.size();
        let Size {
            width: icon_width,
            height: icon_height,
        } = self.shapes.size;
        ctx.transform(Affine::scale_non_uniform(
            width * icon_width.recip(),
            height * icon_height.recip(),
        ));
        let brush = ctx.solid_brush(self.color.clone());
        for shape in self.shapes.shapes {
            ctx.fill(shape, &brush);
        }
    }
}

/// Factories for creating material icons for druid.
///
/// # Examples
///
/// ```
/// # use druid::{Widget, Data, Color, WidgetExt};
/// # #[derive(Data, Clone)]
/// # struct MyData;
/// use druid_material_icons as icons;
/// fn build_ui() -> impl Widget<MyData> {
///     icons::ADD.new(Color::BLACK).fix_width(12.0).center()
/// }
/// ```
#[derive(Debug, Copy, Clone)]
pub struct IconShapes {
    shapes: &'static [IconShape],
    size: Size,
}

impl IconShapes {
    pub fn new(self, color: Color) -> Icon {
        Icon::new(self, color)
    }
}

#[derive(Debug, Copy, Clone)]
enum IconShape {
    PathEls(&'static [PathEl]),
    Circle(Circle),
}

impl Shape for IconShape {
    type BezPathIter = Box<dyn Iterator<Item = PathEl>>; // GATs would be better here.
    fn to_bez_path(&self, tolerance: f64) -> Self::BezPathIter {
        match self {
            IconShape::PathEls(els) => Box::new(els.to_bez_path(tolerance)),
            IconShape::Circle(circle) => Box::new(circle.to_bez_path(tolerance)),
        }
    }
    fn area(&self) -> f64 {
        match self {
            IconShape::PathEls(els) => els.area(),
            IconShape::Circle(circle) => circle.area(),
        }
    }
    fn perimeter(&self, accuracy: f64) -> f64 {
        match self {
            IconShape::PathEls(els) => els.perimeter(accuracy),
            IconShape::Circle(circle) => circle.perimeter(accuracy),
        }
    }
    fn winding(&self, pt: Point) -> i32 {
        match self {
            IconShape::PathEls(els) => els.winding(pt),
            IconShape::Circle(circle) => circle.winding(pt),
        }
    }
    fn bounding_box(&self) -> Rect {
        match self {
            IconShape::PathEls(els) => els.bounding_box(),
            IconShape::Circle(circle) => circle.bounding_box(),
        }
    }

    fn into_bez_path(self, tolerance: f64) -> BezPath {
        match self {
            IconShape::PathEls(els) => els.into_bez_path(tolerance),
            IconShape::Circle(circle) => circle.into_bez_path(tolerance),
        }
    }
    fn as_circle(&self) -> Option<Circle> {
        match self {
            IconShape::PathEls(_) => None,
            IconShape::Circle(circle) => Some(*circle),
        }
    }
    fn as_path_slice(&self) -> Option<&[PathEl]> {
        match self {
            IconShape::PathEls(els) => Some(els),
            IconShape::Circle(_) => None,
        }
    }
}

include!("./icons.rs.in");

/// Returns the max size of these constraints, whilst also constraining the aspect ratio.
///
/// If the given aspect ratio is not possible, then first choose the closest aspect ratio
/// possible, then choose the largest size possible with that ratio.
fn max_with_aspect(bc: &BoxConstraints, aspect_ratio: f64) -> Size {
    // TODO infinity (this might automatically work)
    // If the there is a point on the aspect ratio line with maximum width, it is the solution.
    let y = bc.max().width * aspect_ratio;
    if y >= bc.min().height && y <= bc.max().height {
        return Size {
            width: bc.max().width,
            height: y,
        };
    }
    // If the there is a point on the aspect ratio line with maximum height, it is the solution.
    let x = bc.max().height * aspect_ratio.recip();
    if x >= bc.min().width && x <= bc.max().width {
        return Size {
            width: x,
            height: bc.max().height,
        };
    }
    // There are no solutions inside the constraints, to test to see whether top-right or
    // bottom-left corner is better. We don't need to worry about nonlinearity of aspect ratio,
    // because both points will either have lower or higher aspect ratio.
    let bottom_left = Size {
        width: bc.min().width,
        height: bc.max().height,
    };
    let top_right = Size {
        width: bc.max().width,
        height: bc.min().height,
    };
    if (aspect_ratiof(bottom_left) - aspect_ratio).abs()
        < (aspect_ratiof(top_right) - aspect_ratio).abs()
    {
        bottom_left
    } else {
        top_right
    }
}

fn aspect_ratiof(size: Size) -> f64 {
    size.height * size.width.recip()
}
