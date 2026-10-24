use crate::model::BuildingNode;

pub const DEFAULT_MIN_BUILDING_HEIGHT: u16 = 3;
pub const DEFAULT_MIN_BUILDING_WIDTH: u16 = 8;
pub const DEFAULT_MAX_BUILDING_WIDTH: u16 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingDimension {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkylinePlacement {
    pub node_index: usize,
    pub start_col: usize,
    pub width: u16,
    pub height: u16,
}

/// Computes building height using a logarithmic scale:
/// height = min_height + [ln(1 + size) / ln(1 + max_size)] * (max_height - min_height)
pub fn calculate_building_height(
    size: u64,
    max_size: u64,
    min_height: u16,
    max_height: u16,
) -> u16 {
    if max_height <= min_height {
        return min_height;
    }
    if size == 0 || max_size == 0 {
        return min_height;
    }

    let size_f = size as f64;
    let max_size_f = max_size as f64;

    let log_size = (1.0 + size_f).ln();
    let log_max = (1.0 + max_size_f).ln();

    if log_max <= 0.0 {
        return min_height;
    }

    let ratio = (log_size / log_max).clamp(0.0, 1.0);
    let height = min_height as f64 + ratio * ((max_height - min_height) as f64);
    (height.round() as u16).clamp(min_height, max_height)
}

/// Computes building width based on item name and whether it is a directory
pub fn calculate_building_width(node: &BuildingNode) -> u16 {
    let name_len = node.name.chars().count() as u16;
    let base_width = (name_len + 2).max(DEFAULT_MIN_BUILDING_WIDTH);
    if node.is_dir {
        (base_width + 2).min(DEFAULT_MAX_BUILDING_WIDTH)
    } else {
        base_width.min(DEFAULT_MAX_BUILDING_WIDTH)
    }
}

/// Computes layout positions for all buildings along the street with 1-character street gaps
pub fn layout_skyline(nodes: &[BuildingNode], available_height: u16) -> Vec<SkylinePlacement> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let max_size = nodes.iter().map(|n| n.size).max().unwrap_or(0);
    let max_height = available_height
        .saturating_sub(2)
        .max(DEFAULT_MIN_BUILDING_HEIGHT);
    let min_height = DEFAULT_MIN_BUILDING_HEIGHT.min(max_height);

    let mut placements = Vec::with_capacity(nodes.len());
    let mut current_col = 0;

    for (index, node) in nodes.iter().enumerate() {
        let width = calculate_building_width(node);
        let height = calculate_building_height(node.size, max_size, min_height, max_height);

        placements.push(SkylinePlacement {
            node_index: index,
            start_col: current_col,
            width,
            height,
        });

        current_col += (width as usize) + 1; // 1 space gap between buildings
    }

    placements
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_calculate_building_height_logarithmic() {
        let min_h = 3;
        let max_h = 25;
        let max_size = 10 * 1024 * 1024 * 1024; // 10 GB

        // 10 KB file
        let small_size = 10 * 1024;
        let small_h = calculate_building_height(small_size, max_size, min_h, max_h);

        // 50 MB file
        let mid_size = 50 * 1024 * 1024;
        let mid_h = calculate_building_height(mid_size, max_size, min_h, max_h);

        // 10 GB file (max)
        let max_h_actual = calculate_building_height(max_size, max_size, min_h, max_h);

        // Assertions
        assert!(small_h >= min_h, "Small file must have at least min_h");
        assert!(small_h < mid_h, "Small file must be shorter than mid file");
        assert!(
            mid_h < max_h_actual,
            "Mid file must be shorter than max file"
        );
        assert_eq!(max_h_actual, max_h, "Max size must reach max_h");
    }

    #[test]
    fn test_calculate_building_height_edge_cases() {
        assert_eq!(calculate_building_height(0, 1000, 3, 20), 3);
        assert_eq!(calculate_building_height(100, 0, 3, 20), 3);
        assert_eq!(calculate_building_height(500, 1000, 10, 5), 10);
    }

    #[test]
    fn test_layout_skyline() {
        let nodes = vec![
            BuildingNode::new(
                "small.txt".into(),
                PathBuf::from("small.txt"),
                false,
                false,
                1024,
                vec![],
            ),
            BuildingNode::new(
                "huge.iso".into(),
                PathBuf::from("huge.iso"),
                false,
                false,
                1024 * 1024 * 1024,
                vec![],
            ),
        ];

        let placements = layout_skyline(&nodes, 20);
        assert_eq!(placements.len(), 2);
        assert_eq!(placements[0].start_col, 0);
        assert!(placements[1].start_col > placements[0].width as usize);
        assert!(placements[1].height > placements[0].height);
    }
}
