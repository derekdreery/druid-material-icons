mod find;

use crate::find::find_icons;
use heck::CamelCase;
use roxmltree::{Document, Node};
use std::{
    fmt::{self, Display, Write},
    fs,
    io::Write as IoWrite,
    path::PathBuf,
};

fn main() {
    let mut out = fs::File::create("icons.rs").unwrap();
    writeln!(
        out,
        r#"
        use druid::{{
            Data, Color, Size,
            kurbo::{{Point, PathEl, Circle, Affine}},
            piet::RenderContext,
        }};
    "#
    )
    .unwrap();
    for icon in find_icons() {
        writeln!(out, "{}", icon.implement()).unwrap();
    }
}

#[derive(Debug)]
struct Icon {
    category: String,
    prefix: String,
    size: u32,
}

impl Icon {
    fn path(&self) -> PathBuf {
        format!(
            "../material-design-icons/{}/svg/production/ic_{}_{}px.svg",
            self.category, self.prefix, self.size
        )
        .into()
    }

    fn rust_name(&self) -> String {
        let name = self.prefix.to_camel_case();
        if name.starts_with("3") {
            format!("Three{}", &name[1..])
        } else {
            name
        }
    }

    fn shapes(&self) -> Vec<KurboShape> {
        let raw = fs::read_to_string(&self.path()).unwrap();
        let doc = Document::parse(&raw).unwrap();
        let svg = doc.root_element();
        assert!(svg.has_tag_name("svg"));
        assert_eq!(svg.attribute("width").unwrap(), self.size.to_string());
        assert_eq!(svg.attribute("height").unwrap(), self.size.to_string());
        svg.children().map(|el| KurboShape::from_svg(el)).collect()
    }

    fn implement(&self) -> Implement {
        Implement(self)
    }
}

#[derive(Debug)]
pub enum KurboShape {
    Circle(kurbo::Circle),
    BezPath(kurbo::BezPath),
}

impl KurboShape {
    fn from_svg(input: Node) -> Self {
        match input.tag_name().name() {
            "circle" => {
                let cx = input.attribute("cx").unwrap().parse::<f64>().unwrap();
                let cy = input.attribute("cy").unwrap().parse::<f64>().unwrap();
                let r = input.attribute("r").unwrap().parse::<f64>().unwrap();
                KurboShape::Circle(kurbo::Circle::new((cx, cy), r))
            }
            "path" => {
                let d = input.attribute("d").unwrap();
                KurboShape::BezPath(kurbo::BezPath::from_svg(d).unwrap())
            }
            other => panic!("unrecognised node: {}", other),
        }
    }
}

impl Display for KurboShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KurboShape::Circle(circle) => write!(
                f,
                "Circle {{ center: {}, radius: {:.2} }}",
                KurboPoint(circle.center),
                circle.radius
            ),
            KurboShape::BezPath(path) => {
                f.write_str("&[")?;
                for el in path.iter() {
                    write!(f, "{},", KurboEl(el))?;
                }
                f.write_str("][..]")
            }
        }
    }
}

pub struct KurboPoint(kurbo::Point);

impl Display for KurboPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point {{ x: {:.2}, y: {:.2} }}", self.0.x, self.0.y)
    }
}

pub struct KurboEl(kurbo::PathEl);

impl Display for KurboEl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use kurbo::PathEl;
        match self.0 {
            PathEl::MoveTo(point) => write!(f, "PathEl::MoveTo({})", KurboPoint(point)),
            PathEl::LineTo(point) => write!(f, "PathEl::LineTo({})", KurboPoint(point)),
            PathEl::QuadTo(point1, point2) => write!(
                f,
                "PathEl::QuadTo({}, {})",
                KurboPoint(point1),
                KurboPoint(point2)
            ),
            PathEl::CurveTo(point1, point2, point3) => write!(
                f,
                "PathEl::CurveTo({}, {}, {})",
                KurboPoint(point1),
                KurboPoint(point2),
                KurboPoint(point3)
            ),
            PathEl::ClosePath => f.write_str("PathEl::ClosePath"),
        }
    }
}

pub struct Implement<'a>(&'a Icon);

impl Display for Implement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut paint = String::new();
        for shape in self.0.shapes() {
            writeln!(paint, "ctx.fill({}, &brush);", shape)?;
        }
        write!(
            f,
            r#"
pub struct {icon} {{
    size: f64,
    color: Color,
}}

impl {icon} {{
    pub fn new(color: Color, size: f64) -> Self {{
        Self {{ color, size }}
    }}
}}

impl<T: Data> ::druid::Widget<T> for {icon} {{
    fn event(
        &mut self,
        _ctx: &mut ::druid::EventCtx,
        _event: &::druid::Event,
        _data: &mut T,
        _env: &::druid::Env
    ) {{
        // no events
    }}
    fn lifecycle(
        &mut self,
        _ctx: &mut ::druid::LifeCycleCtx,
        _event: &::druid::LifeCycle,
        _data: &T,
        _env: &::druid::Env
    ) {{
        // no lifecycle
    }}
    fn update(&mut self,
        _ctx: &mut ::druid::UpdateCtx,
        _old_data: &T,
        _data: &T,
        _env: &::druid::Env
    ) {{
        // no update
    }}
    fn layout(
        &mut self,
        _ctx: &mut ::druid::LayoutCtx,
        bc: &::druid::BoxConstraints,
        _data: &T,
        _env: &::druid::Env
    ) -> Size {{
        bc.constrain((self.size, self.size))
    }}
    fn paint(&mut self, ctx: &mut ::druid::PaintCtx, _data: &T, _env: &::druid::Env) {{
        let Size {{ width, height }} = ctx.size();
        ctx.transform(Affine::scale_non_uniform(width / {size}.0, height / {size}.0));
        let brush = ctx.solid_brush(self.color.clone());
        {paint}
    }}
}}
        "#,
            icon = self.0.rust_name(),
            paint = paint,
            size = self.0.size,
        )
    }
}
