use crate::Icon;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, fs};

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

pub(crate) fn find_icons() -> Vec<Icon> {
    let mut icons: HashMap<String, Icon> = HashMap::new();
    for category in ICON_CATEGORIES.iter() {
        for icon in icon_category(category) {
            icons.insert(icon.rust_name(), icon);
        }
    }
    icons.into_iter().map(|(_, icon)| icon).collect()
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
                // TODO handle non-square icons
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
