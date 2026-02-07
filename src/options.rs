use serde::Deserialize;

/// Options for GPX to GeoJSON conversion.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    /// Include elevation as the 3rd coordinate value (default: true)
    #[serde(default = "default_true")]
    pub include_elevation: bool,

    /// Include timestamps in coordinateProperties.times (default: true)
    #[serde(default = "default_true")]
    pub include_time: bool,

    /// Include metadata (name, desc, etc.) in properties (default: true)
    #[serde(default = "default_true")]
    pub include_metadata: bool,

    /// Which GPX element types to convert (default: all)
    #[serde(default)]
    pub types: Option<Vec<GpxElementType>>,

    /// Join track segments into a single MultiLineString (default: false)
    #[serde(default)]
    pub join_track_segments: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            include_elevation: true,
            include_time: true,
            include_metadata: true,
            types: None,
            join_track_segments: false,
        }
    }
}

impl ConvertOptions {
    pub fn should_include(&self, element_type: GpxElementType) -> bool {
        match &self.types {
            None => true,
            Some(types) => types.contains(&element_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpxElementType {
    Waypoint,
    Route,
    Track,
}

fn default_true() -> bool {
    true
}
