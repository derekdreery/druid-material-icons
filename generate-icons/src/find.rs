use heck::CamelCase;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};

static ICON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^ic_(.*)_(\d+)px\.svg$").unwrap());
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

pub fn find_icons() -> impl Iterator<Item = Icon> {
    FindIcons {
        idx: 0,
        current: vec![],
    }
}

// Wouldn't it be nice if generators were a thing.
struct FindIcons {
    idx: usize,
    current: Vec<Icon>,
}

impl Iterator for FindIcons {
    type Item = Icon;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < ICON_CATEGORIES.len() {
            if self.current.is_empty() {
                self.current
                    .extend(icon_category(ICON_CATEGORIES[self.idx]));
                self.idx += 1;
            }
            Some(self.current.pop().unwrap())
        } else {
            None
        }
    }
}

/// Find all the largest icons in the category.
fn icon_category(name: &str) -> impl Iterator<Item = Icon> + 'static {
    let category = name.to_string();
    let mut icons: HashMap<String, u32> = HashMap::new();
    for icon in fs::read_dir(&format!("../material-design-icons/{}/svg/production", name)).unwrap()
    {
        let icon = icon.unwrap().file_name().into_string().unwrap();
        let captures = match ICON_REGEX.captures(&icon) {
            Some(c) => c,
            None => {
                eprintln!("Skipping {:?}", icon);
                continue;
            }
        };
        let (icon_name, size) = (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str().parse::<u32>().unwrap(),
        );
        let icon_entry = icons.entry(icon_name.to_owned()).or_insert(0);
        if *icon_entry < size {
            *icon_entry = size;
        }
    }
    icons.into_iter().map(move |(prefix, size)| Icon {
        category: category.clone(),
        prefix,
        size,
    })
}
