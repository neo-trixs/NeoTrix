pub mod types;
pub mod schools;
pub mod mountains;
pub mod mappings;
pub mod kb;
pub mod query;
pub mod linking;

pub use types::*;
pub use schools::all_schools;
pub use mountains::known_peaks;
pub use mappings::all_mappings;
pub use kb::{safe_insert_node, safe_insert_edge, now};
pub use query::{
    shanhai_stats, shanhai_peaks, shanhai_mappings, shanhai_evidence, shanhai_schools,
    export_geojson, MappingRecord,
};
pub use linking::{infer_shanhai_links, shanhai_edge_count};
