/// Parsed GPX data containing all waypoints, routes, and tracks.
#[derive(Debug, Default)]
pub struct GpxData {
    pub waypoints: Vec<GpxPoint>,
    pub routes: Vec<GpxRoute>,
    pub tracks: Vec<GpxTrack>,
}

/// A single GPX point (used for wpt, rtept, trkpt).
#[derive(Debug, Clone)]
pub struct GpxPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
    pub time: Option<String>,
    pub name: Option<String>,
    pub cmt: Option<String>,
    pub desc: Option<String>,
    pub src: Option<String>,
    pub sym: Option<String>,
    pub point_type: Option<String>,
    pub link: Option<GpxLink>,
}

impl GpxPoint {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self {
            lat,
            lon,
            ele: None,
            time: None,
            name: None,
            cmt: None,
            desc: None,
            src: None,
            sym: None,
            point_type: None,
            link: None,
        }
    }
}

/// A GPX link element.
#[derive(Debug, Clone)]
pub struct GpxLink {
    pub href: String,
    pub text: Option<String>,
    pub link_type: Option<String>,
}

/// A GPX route (<rte>).
#[derive(Debug, Default)]
pub struct GpxRoute {
    pub name: Option<String>,
    pub cmt: Option<String>,
    pub desc: Option<String>,
    pub src: Option<String>,
    pub link: Option<GpxLink>,
    pub number: Option<u32>,
    pub route_type: Option<String>,
    pub points: Vec<GpxPoint>,
}

/// A GPX track (<trk>).
#[derive(Debug, Default)]
pub struct GpxTrack {
    pub name: Option<String>,
    pub cmt: Option<String>,
    pub desc: Option<String>,
    pub src: Option<String>,
    pub link: Option<GpxLink>,
    pub number: Option<u32>,
    pub track_type: Option<String>,
    pub segments: Vec<GpxSegment>,
}

/// A GPX track segment (<trkseg>).
#[derive(Debug, Default)]
pub struct GpxSegment {
    pub points: Vec<GpxPoint>,
}
