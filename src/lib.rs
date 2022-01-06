#[cfg(feature = "druid")]
use druid::{
    kurbo::{Affine, PathEl, Point, Rect, Shape, Size},
    widget::prelude::*,
    Color, Data,
};
#[cfg(not(feature = "druid"))]
use kurbo::{PathEl, Point, Rect, Shape, Size};

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
#[cfg(feature = "druid")]
pub struct Icon {
    paths: IconPaths,
    color: Color,
}

#[cfg(feature = "druid")]
impl Icon {
    #[inline]
    fn new(paths: IconPaths, color: Color) -> Self {
        Self { paths, color }
    }
}

#[cfg(feature = "druid")]
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
        let Size { width, height } = self.paths.size;
        bc.constrain_aspect_ratio(height / width, width)
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, _env: &Env) {
        let Size { width, height } = ctx.size();
        let Size {
            width: icon_width,
            height: icon_height,
        } = self.paths.size;
        ctx.transform(Affine::scale_non_uniform(
            width * icon_width.recip(),
            height * icon_height.recip(),
        ));
        // TODO This makes slightly more brushes than it needs to. Probably not an issue.
        for shape in self.paths.paths {
            let color = self.color.clone();
            let (_, _, _, alpha) = color.as_rgba();
            let color = color.with_alpha(alpha * shape.opacity);
            let brush = ctx.solid_brush(color);
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
pub struct IconPaths {
    pub paths: &'static [IconPath],
    pub size: Size,
}

#[cfg(feature = "druid")]
impl IconPaths {
    pub fn new(self, color: Color) -> Icon {
        Icon::new(self, color)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IconPath {
    els: &'static [PathEl],
    opacity: f64,
}

impl Shape for IconPath {
    type PathElementsIter = std::iter::Copied<std::slice::Iter<'static, PathEl>>;
    fn path_elements(&self, _tolerance: f64) -> Self::PathElementsIter {
        self.els.iter().copied()
    }

    fn area(&self) -> f64 {
        self.els.area()
    }

    fn perimeter(&self, accuracy: f64) -> f64 {
        self.els.perimeter(accuracy)
    }

    fn winding(&self, pt: Point) -> i32 {
        self.els.winding(pt)
    }
    fn bounding_box(&self) -> Rect {
        self.els.bounding_box()
    }

    fn as_path_slice(&self) -> Option<&[PathEl]> {
        Some(self.els)
    }
}

include!("./icons.rs.in");
