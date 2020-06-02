mod find;

use find::find_icons;

const ICON_CATEGORIES: &[&str] = &[
    "action",
    "alert",
    "av",
    "communication",
    "content",
    "device",
    "editor",
    "file",
    "hardware",
    "image",
    "maps",
    "navigation",
    "notification",
    "places",
    "social",
    "toggle",
];

fn main() {
    for image in find_icons() {
        println!("{} => {}", image.path().display(), image.rust_name());
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
        self.prefix.to_camel_case()
    }
}
