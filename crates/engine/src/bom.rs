//! Bill-of-materials computation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::document::Document;

/// One grouped inventory line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BomLine {
    /// Manufacturer as recorded in shape data.
    pub manufacturer: String,
    /// Manufacturer part number.
    pub part_number: String,
    /// Product description.
    pub description: String,
    /// Number of placed units.
    pub quantity: u32,
    /// Distinct locations where the item is installed.
    pub locations: Vec<String>,
    /// Ids of the grouped placed items.
    pub item_ids: Vec<u64>,
}

impl BomLine {
    /// Stable key used to merge two lines.
    #[must_use]
    pub fn identity(&self) -> (String, String, String) {
        (
            self.manufacturer.trim().to_ascii_lowercase(),
            self.part_number.trim().to_ascii_lowercase(),
            self.description.trim().to_ascii_lowercase(),
        )
    }
}

/// A complete bill of materials.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bom {
    /// Grouped lines, sorted by manufacturer, part number, description.
    pub lines: Vec<BomLine>,
    /// Total number of placed items counted.
    pub total_quantity: u32,
}

impl Bom {
    /// Whether the document has no BOM entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Number of distinct part identities.
    #[must_use]
    pub fn distinct_parts(&self) -> usize {
        self.lines.len()
    }
}

impl Document {
    /// Compute the grouped bill of materials.
    ///
    /// Items are grouped by a case-insensitive tuple of manufacturer, part
    /// number and description. The display values come from the first item in
    /// each group, keeping the computation deterministic.
    #[must_use]
    pub fn bom(&self) -> Bom {
        let mut groups: BTreeMap<(String, String, String), BomLine> = BTreeMap::new();
        let mut total_quantity = 0_u32;

        for item in &self.items {
            total_quantity = total_quantity.saturating_add(1);
            let mut line = BomLine {
                manufacturer: item.manufacturer().to_owned(),
                part_number: item.part_number().to_owned(),
                description: item.description().to_owned(),
                quantity: 0,
                locations: Vec::new(),
                item_ids: Vec::new(),
            };
            let key = line.identity();
            let entry = groups.entry(key).or_insert_with(|| {
                line.quantity = 0;
                line
            });
            entry.quantity = entry.quantity.saturating_add(1);
            entry.item_ids.push(item.id);
            let location = item.location_summary();
            if !location.is_empty() && !entry.locations.contains(&location) {
                entry.locations.push(location);
            }
        }

        let mut lines: Vec<BomLine> = groups.into_values().collect();
        for line in &mut lines {
            line.locations.sort();
            line.locations.dedup();
            line.item_ids.sort_unstable();
        }
        lines.sort_by(|a, b| {
            a.manufacturer
                .to_ascii_lowercase()
                .cmp(&b.manufacturer.to_ascii_lowercase())
                .then_with(|| {
                    a.part_number
                        .to_ascii_lowercase()
                        .cmp(&b.part_number.to_ascii_lowercase())
                })
                .then_with(|| {
                    a.description
                        .to_ascii_lowercase()
                        .cmp(&b.description.to_ascii_lowercase())
                })
        });

        Bom {
            lines,
            total_quantity,
        }
    }
}

/// Merge two BOMs, used by future multi-page projects.
#[must_use]
pub fn merge_boms(boms: &[Bom]) -> Bom {
    let mut groups: BTreeMap<(String, String, String), BomLine> = BTreeMap::new();
    let mut total_quantity = 0_u32;
    for bom in boms {
        total_quantity = total_quantity.saturating_add(bom.total_quantity);
        for line in &bom.lines {
            let entry = groups.entry(line.identity()).or_insert_with(|| BomLine {
                quantity: 0,
                ..line.clone()
            });
            entry.quantity = entry.quantity.saturating_add(line.quantity);
            entry.item_ids.extend(line.item_ids.iter().copied());
            for location in &line.locations {
                if !entry.locations.contains(location) {
                    entry.locations.push(location.clone());
                }
            }
        }
    }
    let mut lines: Vec<BomLine> = groups.into_values().collect();
    for line in &mut lines {
        line.item_ids.sort_unstable();
        line.item_ids.dedup();
        line.locations.sort();
        line.locations.dedup();
    }
    lines.sort_by(|a, b| {
        a.manufacturer
            .to_ascii_lowercase()
            .cmp(&b.manufacturer.to_ascii_lowercase())
            .then_with(|| a.part_number.cmp(&b.part_number))
    });
    Bom {
        lines,
        total_quantity,
    }
}

/// Convenience: number of locations represented in a set, used by exporters.
#[must_use]
pub fn unique_locations(lines: &[BomLine]) -> BTreeSet<String> {
    lines
        .iter()
        .flat_map(|line| line.locations.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Item, ProjectFields, StencilRef};
    use crate::geometry::{Point, Size};

    fn item(name: &str, manufacturer: &str, part: &str, description: &str) -> Item {
        let mut item = Item::new(
            StencilRef {
                library: "Test".into(),
                master_id: part.into(),
                name: name.into(),
            },
            name,
            Point::default(),
            Size::new(482.6, 44.45),
        );
        item.set_shape_value("Manufacturer", manufacturer);
        item.set_shape_value("Part Number", part);
        item.set_shape_value("Product Description", description);
        item
    }

    #[test]
    fn groups_by_identity_and_counts_quantity() {
        let mut doc = Document::new("BOM");
        doc.add_item(item("Panel A", "RackCraft", "RC-PP-24", "CAT6 panel"));
        doc.add_item(item("Panel B", "rackcraft", "rc-pp-24", "cat6 panel"));
        doc.add_item(item("Switch", "RackCraft", "RC-SW-24", "24-port switch"));
        let bom = doc.bom();
        assert_eq!(bom.total_quantity, 3);
        assert_eq!(bom.distinct_parts(), 2);
        let panel = bom
            .lines
            .iter()
            .find(|line| line.part_number.eq_ignore_ascii_case("RC-PP-24"))
            .unwrap();
        assert_eq!(panel.quantity, 2);
        assert_eq!(panel.manufacturer, "RackCraft");
        assert_eq!(panel.item_ids.len(), 2);
    }

    #[test]
    fn locations_are_deduplicated_and_sorted() {
        let mut doc = Document::new("Locations");
        for location in ["Room B", "Room A", "Room B"] {
            let mut placed = item("Panel", "RackCraft", "RC-PP-24", "CAT6 panel");
            placed.project = ProjectFields {
                building: "HQ".into(),
                location: location.into(),
                ..ProjectFields::default()
            };
            doc.add_item(placed);
        }
        let bom = doc.bom();
        assert_eq!(bom.lines[0].quantity, 3);
        assert_eq!(
            bom.lines[0].locations,
            vec!["HQ — Room A".to_owned(), "HQ — Room B".to_owned()]
        );
    }

    #[test]
    fn missing_shape_data_has_stable_fallbacks() {
        let mut doc = Document::new("Missing");
        let placed = Item::new(
            StencilRef::default(),
            "Mystery box",
            Point::default(),
            Size::new(10.0, 10.0),
        );
        doc.add_item(placed);
        let bom = doc.bom();
        assert_eq!(bom.lines[0].manufacturer, "Unknown");
        assert_eq!(bom.lines[0].part_number, "");
        assert_eq!(bom.lines[0].description, "Mystery box");
    }

    #[test]
    fn merge_boms_adds_quantities() {
        let mut a = Document::new("A");
        a.add_item(item("Panel", "RackCraft", "RC-PP-24", "CAT6 panel"));
        let mut b = Document::new("B");
        b.add_item(item("Panel", "RackCraft", "RC-PP-24", "CAT6 panel"));
        b.add_item(item("Switch", "RackCraft", "RC-SW-24", "Switch"));
        let merged = merge_boms(&[a.bom(), b.bom()]);
        assert_eq!(merged.total_quantity, 3);
        assert_eq!(merged.lines.len(), 2);
        assert_eq!(merged.lines[0].quantity, 2);
    }
}
