use heck::ShoutySnakeCase;
use once_cell::sync::Lazy;
use qu::ick_use::*;
use regex::Regex;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Write},
    fs,
    io::Write as IoWrite,
    path::{Path, PathBuf},
    sync::Arc,
};
use usvg::Visibility;

static ICON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)px\.svg$").unwrap());
const USE: &str = r#"
use crate::{PathEl, Point, Size, IconPath, IconPaths};
"#;

#[qu::ick]
fn main() -> Result {
    let icons = Icons::load("../material-design-icons")?;
    let mut out = fs::File::create("icons.rs").context("creating `icons.rs`")?;
    for (variant, icons) in icons.0.iter() {
        // We are generating way too much output, which slows down rustc a lot. I would love to
        // hear any suggestions on how to improve this...
        if &**variant != "normal" {
            continue;
        }
        //writeln!(out, "#[cfg(feature = \"{0}\")]\npub mod {0} {{", variant)?;
        writeln!(out, "pub mod {0} {{", variant)?;
        for (category, icons) in icons.iter() {
            //writeln!(out, "#[cfg(feature = \"{0}\")]\npub mod {0} {{", category)?;
            writeln!(out, "pub mod {0} {{", category)?;
            writeln!(out, "{}", USE)?;
            for icon in icons.values() {
                writeln!(out, "{}", icon.implement())?;
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")?;
    }
    Ok(())
}

struct Icons(BTreeMap<Arc<str>, BTreeMap<Arc<str>, BTreeMap<Arc<str>, Icon>>>);

impl Icons {
    /// Load all found icons into memory.
    fn load(root: impl AsRef<Path>) -> Result<Self> {
        let mut icons = Icons(BTreeMap::new());
        let root = root.as_ref().join("src");
        for entry in fs::read_dir(&root).context("reading root directory")? {
            let entry = entry?;
            let category: Arc<str> = entry
                .file_name()
                .into_string()
                .ok()
                .context("non-utf8 category filename")?
                .into();
            for entry in fs::read_dir(&entry.path())? {
                let entry = entry?;
                let name: Arc<str> = entry
                    .file_name()
                    .into_string()
                    .ok()
                    .context("non-utf8 icon filename")?
                    .into();
                for entry in fs::read_dir(&entry.path())? {
                    let entry = entry?;
                    let variant = entry
                        .file_name()
                        .into_string()
                        .ok()
                        .context("non-utf8 variant filename")?;
                    let variant = variant
                        .strip_prefix("materialicons")
                        .context("unexpected variant format")?;
                    let variant: Arc<str> = if variant.is_empty() {
                        "normal"
                    } else {
                        variant
                    }
                    .into();
                    for entry in fs::read_dir(&entry.path())? {
                        let entry = entry?;
                        let filename = entry
                            .file_name()
                            .into_string()
                            .ok()
                            .context("non-utf8 file filename")?;
                        let captures = ICON_REGEX
                            .captures(&filename)
                            .context("icon filename not in expected format")?;
                        let size = captures
                            .get(1)
                            .unwrap()
                            .as_str()
                            .parse::<f64>()
                            .context("parsing icon size")?;
                        let path = entry.path();
                        log::trace!("loading icon {}", path.display());
                        let icon = Icon::from_path(
                            &path,
                            category.clone(),
                            name.clone(),
                            variant.clone(),
                            size,
                        )
                        .with_context(|| format!("loading icon {}", path.display()))?;
                        icons
                            .0
                            .entry(variant.clone())
                            .or_default()
                            .entry(category.clone())
                            .or_default()
                            .insert(name.clone(), icon);
                    }
                }
            }
        }
        Ok(icons)
    }
}

#[derive(Debug)]
struct Icon {
    category: Arc<str>,
    name: Arc<str>,
    variant: Arc<str>,
    size: f64,
    paths: Vec<OpacityPath>,
}

impl Icon {
    fn from_path(
        path: &Path,
        category: Arc<str>,
        name: Arc<str>,
        variant: Arc<str>,
        size: f64,
    ) -> Result<Self> {
        let raw = fs::read(path).context(format!("reading {}", path.display()))?;
        let opts = usvg::Options::default();
        let doc = usvg::Tree::from_data(&raw, &opts.to_ref())?;
        //println!("{:?}", doc.to_string(&Default::default()));
        let mut children = doc.root().children();
        let defs = children.next().context("expected children")?;
        if defs.has_children() {
            log::warn!(
                "ignoring defs, will probably output incorrect icon ({}/{}/{})",
                category,
                name,
                variant
            );
        }

        let mut paths = vec![];
        let mut transform = vec![];
        for child in children {
            handle_child(child, &mut transform, 1., &mut paths)?;
        }
        Ok(Self {
            category,
            name,
            variant,
            size,
            paths,
        })
    }

    fn const_name(&self) -> String {
        let name = self.name.to_shouty_snake_case();
        if matches!(name.chars().next(), Some(d) if d.is_digit(10)) {
            format!("_{}", name)
        } else {
            name
        }
    }

    fn implement(&self) -> Implement {
        Implement(self)
    }
}

/// Function to be called recursively to extract paths from svg.
fn handle_child(
    node: usvg::Node,
    transform: &mut Vec<kurbo::Affine>,
    mut opacity: f64,
    paths: &mut Vec<OpacityPath>,
) -> Result {
    match &*node.borrow() {
        usvg::NodeKind::Path(path) => {
            if let Some(mut path) = handle_path(path) {
                for aff in transform.iter().rev() {
                    path = *aff * path;
                }
                paths.push(OpacityPath { path, opacity });
            }
        }
        usvg::NodeKind::Group(group) => {
            let (aff, opacity_change) = handle_group(group)?;
            if let Some(aff) = aff {
                transform.push(aff);
            }
            if let Some(op) = opacity_change {
                opacity *= op
            }
            for child in node.children() {
                handle_child(child, transform, opacity, paths)?;
            }
            if aff.is_some() {
                transform.pop();
            }
        }
        other => log::warn!("unexpected node type {:?}", other),
    }
    Ok(())
}

/// Check that the group makes no difference
fn handle_group(input: &usvg::Group) -> Result<(Option<kurbo::Affine>, Option<f64>)> {
    ensure!(input.id == "");
    // transform below
    let transform = if input.transform != usvg::Transform::new(1., 0., 0., 1., 0., 0.) {
        let t = input.transform;
        Some(kurbo::Affine::new([t.a, t.b, t.c, t.d, t.e, t.f]))
    } else {
        None
    };
    let opacity = if input.opacity.value() != 1. {
        Some(input.opacity.value())
    } else {
        None
    };
    if input.clip_path.is_some() {
        log::warn!("unhandled clip path");
    }
    ensure!(input.mask.is_none());
    ensure!(input.filter.is_empty());
    ensure!(input.filter_fill.is_none());
    ensure!(input.filter_stroke.is_none());
    // ignore enable_background

    Ok((transform, opacity))
}

fn handle_path(input: &usvg::Path) -> Option<kurbo::BezPath> {
    if matches!(input.visibility, Visibility::Hidden) || input.fill.is_none() {
        return None;
    }
    let mut bez_path = kurbo::BezPath::new();
    for segment in input.data.0.iter().cloned() {
        match segment {
            usvg::PathSegment::MoveTo { x, y } => bez_path.move_to((x, y)),
            usvg::PathSegment::LineTo { x, y } => bez_path.line_to((x, y)),
            usvg::PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => bez_path.curve_to((x1, y1), (x2, y2), (x, y)),
            usvg::PathSegment::ClosePath => bez_path.close_path(),
        }
    }
    Some(bez_path)
}

#[derive(Debug)]
pub struct OpacityPath {
    path: kurbo::BezPath,
    opacity: f64,
}

impl Display for OpacityPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("IconPath { els: &[")?;
        for el in self.path.iter() {
            write!(f, "{},", KurboEl(el))?;
        }
        write!(f, "], opacity: {:.2} }}", self.opacity)
    }
}

pub struct KurboPoint(kurbo::Point);

impl Display for KurboPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point {{ x: {:.2}, y: {:.2} }}", self.0.x, self.0.y)
    }
}

pub struct KurboSize(kurbo::Size);

impl Display for KurboSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Size {{ width: {:.2}, height: {:.2} }}",
            self.0.width, self.0.height
        )
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
        let mut paths = String::new();
        for path in &self.0.paths {
            writeln!(paths, "{},", path)?;
        }
        write!(
            f,
            r#"
pub const {}: IconPaths = IconPaths {{
    paths: &[{}],
    size: {},
}};
        "#,
            self.0.const_name(),
            paths,
            KurboSize(kurbo::Size::new(self.0.size, self.0.size))
        )
    }
}

pub struct MaterialSize(kurbo::Size);

impl Display for MaterialSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.width == self.0.height {
            write!(f, "{}", self.0.width)
        } else {
            write!(f, "{}x{}", self.0.width, self.0.height)
        }
    }
}
