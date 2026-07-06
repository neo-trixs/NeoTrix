use crate::neotrix::nt_shanhai_geo::types::*;

pub fn all_schools() -> Vec<SchoolParameters> {
    vec![
        SchoolParameters {
            name: "华夏说——谭其骧学术体系".into(),
            founder: "谭其骧".into(),
            description: "《山海经》地理范围不出华夏，447座山中约140座可确切定位在豫西/晋南/陕中".into(),
            li_scale: LiScale::QinHan,
            scope: GeographicScope::ChinaOnly,
            key_mountains: vec![
                "华山".into(), "太行山".into(), "王屋山".into(), "嵩山".into(),
            ],
            confidence_base: 0.9,
        },
        SchoolParameters {
            name: "世界圈说——宫玉海学术体系".into(),
            founder: "宫玉海".into(),
            description: "《山海经》描述全球地理：昆仑=非洲，东山经=北美，轩辕之国=匈牙利".into(),
            li_scale: LiScale::LiuZongdi,
            scope: GeographicScope::Global,
            key_mountains: vec![
                "昆仑山".into(), "不周山".into(), "炎火山".into(),
            ],
            confidence_base: 0.75,
        },
        SchoolParameters {
            name: "局部小区说——山东/云南中心论".into(),
            founder: "何幼琦/扶永发".into(),
            description: "《山海经》地理限于山东半岛(何幼琦)或云南西部(扶永发)".into(),
            li_scale: LiScale::Zhou,
            scope: GeographicScope::LocalRegion("山东半岛/云南西部".into()),
            key_mountains: vec![],
            confidence_base: 0.75,
        },
    ]
}
