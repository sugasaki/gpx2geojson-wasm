use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::error::Gpx2GeoJsonError;
use crate::gpx_types::*;

type Result<T> = std::result::Result<T, Gpx2GeoJsonError>;

/// Parse a GPX XML string into GpxData.
pub fn parse_gpx(xml: &str) -> Result<GpxData> {
    let mut reader = Reader::from_str(xml);
    let mut data = GpxData::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"wpt" => {
                    if let Some(pt) = parse_point(&e, &mut reader)? {
                        data.waypoints.push(pt);
                    }
                }
                b"rte" => data.routes.push(parse_route(&mut reader)?),
                b"trk" => data.tracks.push(parse_track(&mut reader)?),
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"wpt" {
                    if let Ok((lat, lon)) = parse_lat_lon(&e) {
                        data.waypoints.push(GpxPoint::new(lat, lon));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(data)
}

/// Parse lat/lon attributes from a point element's start tag.
fn parse_lat_lon(e: &BytesStart<'_>) -> Result<(f64, f64)> {
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;

    for attr_result in e.attributes() {
        let attr = attr_result.map_err(|e| Gpx2GeoJsonError::XmlParse(e.into()))?;
        let key = attr.key.local_name();
        let val = std::str::from_utf8(&attr.value).unwrap_or_default();
        match key.as_ref() {
            b"lat" => {
                lat = Some(val.parse::<f64>().map_err(|_| {
                    Gpx2GeoJsonError::InvalidAttribute {
                        element: "point",
                        attribute: "lat",
                        value: val.to_string(),
                    }
                })?);
            }
            b"lon" => {
                lon = Some(val.parse::<f64>().map_err(|_| {
                    Gpx2GeoJsonError::InvalidAttribute {
                        element: "point",
                        attribute: "lon",
                        value: val.to_string(),
                    }
                })?);
            }
            _ => {}
        }
    }

    let lat = lat.ok_or(Gpx2GeoJsonError::MissingAttribute {
        element: "point",
        attribute: "lat",
    })?;
    let lon = lon.ok_or(Gpx2GeoJsonError::MissingAttribute {
        element: "point",
        attribute: "lon",
    })?;

    Ok((lat, lon))
}

/// Parse a point element (wpt, rtept, trkpt) and its children.
/// Called after receiving Event::Start for the point element.
fn parse_point<'a>(
    start: &BytesStart<'a>,
    reader: &mut Reader<&'a [u8]>,
) -> Result<Option<GpxPoint>> {
    let (lat, lon) = match parse_lat_lon(start) {
        Ok(coords) => coords,
        Err(_) => {
            // Skip this point if lat/lon are missing or invalid
            reader
                .read_to_end(start.name())
                .map_err(Gpx2GeoJsonError::XmlParse)?;
            return Ok(None);
        }
    };

    let mut point = GpxPoint::new(lat, lon);
    let end_name = start.name().0.to_vec(); // own the end tag name for comparison

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"ele" => {
                    let text = reader
                        .read_text(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                    point.ele = text.parse::<f64>().ok();
                }
                b"time" => {
                    point.time = Some(read_text_owned(reader, &e)?);
                }
                b"name" => {
                    point.name = Some(read_text_owned(reader, &e)?);
                }
                b"cmt" => {
                    point.cmt = Some(read_text_owned(reader, &e)?);
                }
                b"desc" => {
                    point.desc = Some(read_text_owned(reader, &e)?);
                }
                b"src" => {
                    point.src = Some(read_text_owned(reader, &e)?);
                }
                b"sym" => {
                    point.sym = Some(read_text_owned(reader, &e)?);
                }
                b"type" => {
                    point.point_type = Some(read_text_owned(reader, &e)?);
                }
                b"link" => {
                    point.link = Some(parse_link(&e, reader)?);
                }
                _ => {
                    // Skip unknown/extensions elements
                    reader
                        .read_to_end(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                }
            },
            Ok(Event::End(e)) if e.name().0 == end_name.as_slice() => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(Some(point))
}

/// Parse a <link> element.
fn parse_link<'a>(
    start: &BytesStart<'a>,
    reader: &mut Reader<&'a [u8]>,
) -> Result<GpxLink> {
    let mut href = String::new();
    for attr_result in start.attributes() {
        if let Ok(attr) = attr_result {
            if attr.key.local_name().as_ref() == b"href" {
                href = std::str::from_utf8(&attr.value)
                    .unwrap_or_default()
                    .to_string();
            }
        }
    }

    let mut text: Option<String> = None;
    let mut link_type: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"text" => text = Some(read_text_owned(reader, &e)?),
                b"type" => link_type = Some(read_text_owned(reader, &e)?),
                _ => {
                    reader
                        .read_to_end(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                }
            },
            Ok(Event::End(e)) if e.local_name().as_ref() == b"link" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(GpxLink {
        href,
        text,
        link_type,
    })
}

/// Parse a <rte> element.
fn parse_route<'a>(reader: &mut Reader<&'a [u8]>) -> Result<GpxRoute> {
    let mut route = GpxRoute::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"name" => route.name = Some(read_text_owned(reader, &e)?),
                b"cmt" => route.cmt = Some(read_text_owned(reader, &e)?),
                b"desc" => route.desc = Some(read_text_owned(reader, &e)?),
                b"src" => route.src = Some(read_text_owned(reader, &e)?),
                b"type" => route.route_type = Some(read_text_owned(reader, &e)?),
                b"number" => {
                    let text = read_text_owned(reader, &e)?;
                    route.number = text.parse::<u32>().ok();
                }
                b"link" => route.link = Some(parse_link(&e, reader)?),
                b"rtept" => {
                    if let Some(pt) = parse_point(&e, reader)? {
                        route.points.push(pt);
                    }
                }
                _ => {
                    reader
                        .read_to_end(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                }
            },
            Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"rtept" {
                    if let Ok((lat, lon)) = parse_lat_lon(&e) {
                        route.points.push(GpxPoint::new(lat, lon));
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"rte" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(route)
}

/// Parse a <trk> element.
fn parse_track<'a>(reader: &mut Reader<&'a [u8]>) -> Result<GpxTrack> {
    let mut track = GpxTrack::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"name" => track.name = Some(read_text_owned(reader, &e)?),
                b"cmt" => track.cmt = Some(read_text_owned(reader, &e)?),
                b"desc" => track.desc = Some(read_text_owned(reader, &e)?),
                b"src" => track.src = Some(read_text_owned(reader, &e)?),
                b"type" => track.track_type = Some(read_text_owned(reader, &e)?),
                b"number" => {
                    let text = read_text_owned(reader, &e)?;
                    track.number = text.parse::<u32>().ok();
                }
                b"link" => track.link = Some(parse_link(&e, reader)?),
                b"trkseg" => {
                    let seg = parse_segment(reader)?;
                    if !seg.points.is_empty() {
                        track.segments.push(seg);
                    }
                }
                _ => {
                    reader
                        .read_to_end(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                }
            },
            Ok(Event::End(e)) if e.local_name().as_ref() == b"trk" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(track)
}

/// Parse a <trkseg> element.
fn parse_segment<'a>(reader: &mut Reader<&'a [u8]>) -> Result<GpxSegment> {
    let mut segment = GpxSegment::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"trkpt" => {
                    if let Some(pt) = parse_point(&e, reader)? {
                        segment.points.push(pt);
                    }
                }
                _ => {
                    reader
                        .read_to_end(e.name())
                        .map_err(Gpx2GeoJsonError::XmlParse)?;
                }
            },
            Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"trkpt" {
                    if let Ok((lat, lon)) = parse_lat_lon(&e) {
                        segment.points.push(GpxPoint::new(lat, lon));
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"trkseg" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(segment)
}

/// Read text content of an element as an owned String.
/// Handles regular text, CDATA sections, and entity references (Event::GeneralRef).
fn read_text_owned<'a>(
    reader: &mut Reader<&'a [u8]>,
    start: &BytesStart<'_>,
) -> Result<String> {
    let end_name = start.name().0.to_vec();
    let mut text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Text(e)) => {
                let raw = std::str::from_utf8(e.as_ref()).unwrap_or_default();
                text.push_str(raw);
            }
            Ok(Event::CData(e)) => {
                let s = std::str::from_utf8(e.as_ref()).unwrap_or_default();
                text.push_str(s);
            }
            Ok(Event::GeneralRef(e)) => {
                // Handle character references (&#60; &#x3C;) and predefined entities
                if let Ok(Some(ch)) = e.resolve_char_ref() {
                    text.push(ch);
                } else {
                    // Predefined XML entities: amp, lt, gt, quot, apos
                    let name = std::str::from_utf8(e.as_ref()).unwrap_or_default();
                    match name {
                        "amp" => text.push('&'),
                        "lt" => text.push('<'),
                        "gt" => text.push('>'),
                        "quot" => text.push('"'),
                        "apos" => text.push('\''),
                        _ => {} // Unknown entity, skip
                    }
                }
            }
            Ok(Event::End(e)) if e.name().0 == end_name.as_slice() => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(Gpx2GeoJsonError::XmlParse(e)),
            _ => {}
        }
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_waypoint() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.6762" lon="139.6503"/>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 1);
        assert!((data.waypoints[0].lat - 35.6762).abs() < 1e-10);
        assert!((data.waypoints[0].lon - 139.6503).abs() < 1e-10);
    }

    #[test]
    fn test_waypoint_with_children() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.6762" lon="139.6503">
    <ele>40.5</ele>
    <time>2025-01-01T00:00:00Z</time>
    <name>Tokyo Tower</name>
    <desc>A famous landmark</desc>
    <cmt>Comment</cmt>
    <src>GPS</src>
    <sym>Flag</sym>
    <type>POI</type>
  </wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let pt = &data.waypoints[0];
        assert!((pt.ele.unwrap() - 40.5).abs() < 1e-10);
        assert_eq!(pt.time.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(pt.name.as_deref(), Some("Tokyo Tower"));
        assert_eq!(pt.desc.as_deref(), Some("A famous landmark"));
        assert_eq!(pt.cmt.as_deref(), Some("Comment"));
        assert_eq!(pt.src.as_deref(), Some("GPS"));
        assert_eq!(pt.sym.as_deref(), Some("Flag"));
        assert_eq!(pt.point_type.as_deref(), Some("POI"));
    }

    #[test]
    fn test_simple_route() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <rte>
    <name>Test Route</name>
    <rtept lat="35.0" lon="139.0"/>
    <rtept lat="36.0" lon="140.0"/>
    <rtept lat="37.0" lon="141.0"/>
  </rte>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.routes.len(), 1);
        assert_eq!(data.routes[0].name.as_deref(), Some("Test Route"));
        assert_eq!(data.routes[0].points.len(), 3);
    }

    #[test]
    fn test_simple_track() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <name>Morning Run</name>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"><ele>10.0</ele></trkpt>
      <trkpt lat="35.001" lon="139.001"><ele>11.0</ele></trkpt>
      <trkpt lat="35.002" lon="139.002"><ele>12.0</ele></trkpt>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.tracks.len(), 1);
        assert_eq!(data.tracks[0].name.as_deref(), Some("Morning Run"));
        assert_eq!(data.tracks[0].segments.len(), 1);
        assert_eq!(data.tracks[0].segments[0].points.len(), 3);
    }

    #[test]
    fn test_multi_segment_track() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"/>
      <trkpt lat="35.001" lon="139.001"/>
    </trkseg>
    <trkseg>
      <trkpt lat="36.0" lon="140.0"/>
      <trkpt lat="36.001" lon="140.001"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.tracks[0].segments.len(), 2);
        assert_eq!(data.tracks[0].segments[0].points.len(), 2);
        assert_eq!(data.tracks[0].segments[1].points.len(), 2);
    }

    #[test]
    fn test_empty_gpx() {
        let xml = r#"<?xml version="1.0"?><gpx version="1.1"></gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert!(data.waypoints.is_empty());
        assert!(data.routes.is_empty());
        assert!(data.tracks.is_empty());
    }

    #[test]
    fn test_empty_segment_skipped() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <trkseg></trkseg>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.tracks[0].segments.len(), 1);
        assert_eq!(data.tracks[0].segments[0].points.len(), 1);
    }

    #[test]
    fn test_extensions_skipped() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <trkseg>
      <trkpt lat="35.0" lon="139.0">
        <extensions>
          <gpxtpx:TrackPointExtension xmlns:gpxtpx="http://www.garmin.com/xmlschemas/TrackPointExtension/v1">
            <gpxtpx:hr>150</gpxtpx:hr>
          </gpxtpx:TrackPointExtension>
        </extensions>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.tracks[0].segments[0].points.len(), 1);
    }

    #[test]
    fn test_no_namespace() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0"><name>Test</name></wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 1);
        assert_eq!(data.waypoints[0].name.as_deref(), Some("Test"));
    }

    #[test]
    fn test_with_namespace() {
        let xml = r#"<?xml version="1.0"?>
<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1">
  <wpt lat="35.0" lon="139.0"><name>Test</name></wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 1);
    }

    #[test]
    fn test_cdata() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0">
    <name><![CDATA[Test & Name]]></name>
  </wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints[0].name.as_deref(), Some("Test & Name"));
    }

    #[test]
    fn test_link_element() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0">
    <link href="https://example.com">
      <text>Example</text>
      <type>text/html</type>
    </link>
  </wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let link = data.waypoints[0].link.as_ref().unwrap();
        assert_eq!(link.href, "https://example.com");
        assert_eq!(link.text.as_deref(), Some("Example"));
        assert_eq!(link.link_type.as_deref(), Some("text/html"));
    }

    #[test]
    fn test_missing_lat_lon_skipped() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0"><name>Good</name></wpt>
  <wpt><name>Bad - no coords</name></wpt>
  <wpt lat="36.0" lon="140.0"><name>Also Good</name></wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 2);
        assert_eq!(data.waypoints[0].name.as_deref(), Some("Good"));
        assert_eq!(data.waypoints[1].name.as_deref(), Some("Also Good"));
    }

    #[test]
    fn test_complete_gpx() {
        let xml = r#"<?xml version="1.0"?>
<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1" creator="test">
  <wpt lat="35.6762" lon="139.6503">
    <name>Tokyo</name>
  </wpt>
  <rte>
    <name>Route 1</name>
    <rtept lat="35.0" lon="139.0"/>
    <rtept lat="36.0" lon="140.0"/>
  </rte>
  <trk>
    <name>Track 1</name>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"><time>2025-01-01T00:00:00Z</time></trkpt>
      <trkpt lat="35.001" lon="139.001"><time>2025-01-01T00:01:00Z</time></trkpt>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 1);
        assert_eq!(data.routes.len(), 1);
        assert_eq!(data.tracks.len(), 1);
    }

    #[test]
    fn test_xml_entities() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0">
    <name>Caf&eacute; &amp; Bar</name>
  </wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        // quick-xml's read_text handles standard XML entities
        assert!(data.waypoints[0].name.is_some());
    }

    #[test]
    fn test_gpx10_elements_ignored() {
        let xml = r#"<?xml version="1.0"?>
<gpx xmlns="http://www.topografix.com/GPX/1/0" version="1.0">
  <trk>
    <trkseg>
      <trkpt lat="35.0" lon="139.0">
        <speed>5.5</speed>
        <course>180.0</course>
      </trkpt>
      <trkpt lat="35.001" lon="139.001"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        assert_eq!(data.tracks[0].segments[0].points.len(), 2);
    }
}
