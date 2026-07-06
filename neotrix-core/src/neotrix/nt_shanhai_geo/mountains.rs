use crate::neotrix::nt_shanhai_geo::types::*;

/// Mountains that have been identified with moderate-to-high confidence
/// by one or more scholarly schools.
pub fn known_peaks() -> Vec<MountainPeak> {
    vec![
        // ── 南山经 (South Mountains) ──────────────────────────────
        MountainPeak {
            id: "south-01".into(),
            range_id: "south-1".into(),
            name: "招摇之山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(29.0, 110.0)),
            identification_confidence: 0.6,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.6 },
            ],
        },
        MountainPeak {
            id: "south-02".into(),
            range_id: "south-1".into(),
            name: "堂庭之山".into(),
            position: 2,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(29.5, 109.5)),
            identification_confidence: 0.5,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.5 },
            ],
        },

        // ── 西山经 (West Mountains) ──────────────────────────────
        MountainPeak {
            id: "west-01".into(),
            range_id: "west-1".into(),
            name: "华山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(34.47, 110.08)),
            identification_confidence: 1.0,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 1.0 },
                SchoolRef { school: "世界圈说".into(), scholar: "宫玉海".into(), confidence: 0.9 },
            ],
        },
        MountainPeak {
            id: "west-02".into(),
            range_id: "west-1".into(),
            name: "嶓冢之山".into(),
            position: 9,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(33.5, 105.5)),
            identification_confidence: 0.9,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.9 },
            ],
        },
        MountainPeak {
            id: "west-03".into(),
            range_id: "west-2".into(),
            name: "昆仑之丘".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(39.5, 109.0)),
            identification_confidence: 0.85,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "王红旗".into(), confidence: 0.85 },
            ],
        },
        MountainPeak {
            id: "west-04".into(),
            range_id: "west-2".into(),
            name: "昆仑之丘 (非洲)".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(0.0, 37.0)),
            identification_confidence: 0.75,
            attributed_by: vec![
                SchoolRef { school: "世界圈说".into(), scholar: "宫玉海".into(), confidence: 0.75 },
            ],
        },
        MountainPeak {
            id: "west-05".into(),
            range_id: "west-3".into(),
            name: "不周之山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(40.0, 112.0)),
            identification_confidence: 0.6,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.6 },
            ],
        },
        MountainPeak {
            id: "west-06".into(),
            range_id: "west-3".into(),
            name: "不周山 (非洲)".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(1.0, 36.0)),
            identification_confidence: 0.7,
            attributed_by: vec![
                SchoolRef { school: "世界圈说".into(), scholar: "宫玉海".into(), confidence: 0.7 },
            ],
        },

        // ── 北山经 (North Mountains) ──────────────────────────────
        MountainPeak {
            id: "north-01".into(),
            range_id: "north-1".into(),
            name: "太行山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(38.0, 113.0)),
            identification_confidence: 1.0,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 1.0 },
            ],
        },
        MountainPeak {
            id: "north-02".into(),
            range_id: "north-1".into(),
            name: "王屋山".into(),
            position: 2,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(35.2, 112.2)),
            identification_confidence: 1.0,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 1.0 },
            ],
        },
        MountainPeak {
            id: "north-03".into(),
            range_id: "north-2".into(),
            name: "雁门山".into(),
            position: 5,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(39.0, 112.5)),
            identification_confidence: 0.9,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.9 },
            ],
        },

        // ── 东山经 (East Mountains) ──────────────────────────────
        MountainPeak {
            id: "east-01".into(),
            range_id: "east-1".into(),
            name: "櫔䔄之山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(36.0, 117.0)),
            identification_confidence: 0.6,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.6 },
            ],
        },
        MountainPeak {
            id: "east-02".into(),
            range_id: "east-1".into(),
            name: "东山经→落基山脉".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(39.0, -106.0)),
            identification_confidence: 0.7,
            attributed_by: vec![
                SchoolRef {
                    school: "世界圈说".into(),
                    scholar: "Henriette Mertz".into(),
                    confidence: 0.7,
                },
            ],
        },
        MountainPeak {
            id: "east-03".into(),
            range_id: "east-4".into(),
            name: "光华之谷".into(),
            position: 4,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(36.0, -112.0)),
            identification_confidence: 0.65,
            attributed_by: vec![
                SchoolRef {
                    school: "世界圈说".into(),
                    scholar: "Henriette Mertz".into(),
                    confidence: 0.65,
                },
            ],
        },

        // ── 中山经 (Central Mountains) ────────────────────────────
        MountainPeak {
            id: "central-01".into(),
            range_id: "central-1".into(),
            name: "嵩山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(34.5, 113.0)),
            identification_confidence: 1.0,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 1.0 },
            ],
        },
        MountainPeak {
            id: "central-02".into(),
            range_id: "central-3".into(),
            name: "首阳之山".into(),
            position: 1,
            shanhai_coord: None,
            modern_location: Some(GeoCoord::new(35.0, 111.0)),
            identification_confidence: 0.8,
            attributed_by: vec![
                SchoolRef { school: "华夏说".into(), scholar: "谭其骧".into(), confidence: 0.8 },
            ],
        },
    ]
}
