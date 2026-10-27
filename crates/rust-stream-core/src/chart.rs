//! Maryland CHART camera catalog decoder. No network I/O.
//! A catalog listing must never automatically become a playable source.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartCamera {
    pub id: String,
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub operational: bool,
    /// Candidate URL is untrusted input and MUST pass connection-time policy.
    pub candidate_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum ChartError {
    #[error("invalid CHART JSON")]
    Json(#[from] serde_json::Error),
    #[error("missing camera list in CHART response")]
    MissingList,
    #[error("too many records in camera response")]
    TooManyRecords,
}

/// Parse JSON only. No camera is labelled live or validated by this function.
pub fn parse_chart_cameras(bytes: &[u8]) -> Result<Vec<ChartCamera>, ChartError> {
    const MAX_BYTES: usize = 8 * 1024 * 1024;
    const MAX_RECORDS: usize = 20_000;
    if bytes.len() > MAX_BYTES { return Err(ChartError::TooManyRecords); }
    let value: Value = serde_json::from_slice(bytes)?;
    let list = if let Some(arr) = value.as_array() { Some(arr) } else {
        ["cameras", "Cameras", "cameraData", "data", "results"]
            .iter().find_map(|k| value.get(*k).and_then(Value::as_array))
    }.ok_or(ChartError::MissingList)?;
    if list.len() > MAX_RECORDS { return Err(ChartError::TooManyRecords); }
    Ok(list.iter().filter_map(parse_record).collect())
}

fn get_string<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| v.get(*k).and_then(Value::as_str))
}
fn coordinate(v: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| {
        let n = v.get(*k)?;
        n.as_f64().or_else(|| n.as_str().and_then(|s| s.parse().ok()))
    })
}
fn parse_record(v: &Value) -> Option<ChartCamera> {
    let id = get_string(v, &["id", "cameraId", "cameraID"])?.trim();
    if id.is_empty() || id.len() > 100 { return None; }
    let latitude = coordinate(v, &["lat", "latitude"])?;
    let longitude = coordinate(v, &["lon", "lng", "longitude"])?;
    if !latitude.is_finite() || !longitude.is_finite()
        || !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return None;
    }
    let label = get_string(v, &["description", "name"]).unwrap_or("Maryland traffic camera");
    let label = label.trim().chars().take(160).collect();
    let status = get_string(v, &["opStatus", "operationalStatus"]).unwrap_or("UNKNOWN");
    let operational = status.eq_ignore_ascii_case("OK");
    let candidate_url = get_string(v, &["publicVideoURL", "publicVideoUrl"])
        .filter(|u| u.len() <= 2048)
        .filter(|u| !u.chars().any(char::is_control))
        .map(str::to_owned);
    Some(ChartCamera {
        id: id.to_owned(), label, latitude, longitude, operational, candidate_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_candidate_but_does_not_assert_live() {
        let data = br#"[{"id":"C-100","description":"I-95 at exit",
          "lat":39.1,"lon":-76.9,"opStatus":"OK",
          "publicVideoURL":"https://video.example.net/cam.m3u8"}]"#;
        let cameras = parse_chart_cameras(data).unwrap();
        assert_eq!(cameras.len(), 1);
        assert!(cameras[0].operational);
        assert_eq!(cameras[0].candidate_url.as_deref(), Some("https://video.example.net/cam.m3u8"));
    }
    #[test]
    fn invalid_coordinates_and_ids_are_dropped() {
        let data = br#"{"cameras":[{"id":"a","lat":999,"lon":1},
                   {"id":"b","lat":"39.4","lon":"-76.5","opStatus":"COMM_FAILURE"}]}"#;
        let cameras = parse_chart_cameras(data).unwrap();
        assert_eq!(cameras.len(), 1);
        assert!(!cameras[0].operational);
    }
    #[test]
    fn missing_list_is_error() {
        assert!(matches!(parse_chart_cameras(br#"{}"#), Err(ChartError::MissingList)));
    }
}
