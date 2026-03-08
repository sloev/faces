use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single vertex in the facial mesh.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

/// A single frame of animation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    /// Timestamp in seconds.
    pub timestamp: f32,
    /// List of vertices for this frame.
    pub vertices: Vec<Vertex>,
}

/// A named animation sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    pub name: String,
    pub frames: Vec<Frame>,
}

/// The complete avatar data exported by the Generator and read by the Player.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvatarData {
    /// Mapping of clip names to animation sequences.
    pub clips: HashMap<String, Clip>,
    /// The indices for Delaunay triangulation (static for all frames).
    pub mesh_indices: Vec<u32>,
    /// Base UV coordinates for the texture atlas (0.0 to 1.0).
    pub base_uvs: Vec<Vertex>,
}

impl AvatarData {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let mut clips = HashMap::new();
        clips.insert(
            "idle".to_string(),
            Clip {
                name: "idle".to_string(),
                frames: vec![Frame {
                    timestamp: 0.0,
                    vertices: vec![Vertex { x: 0.1, y: 0.2 }],
                }],
            },
        );

        let data = AvatarData {
            clips,
            mesh_indices: vec![0, 1, 2],
            base_uvs: vec![Vertex { x: 0.5, y: 0.5 }],
        };

        let serialized = data.to_json().unwrap();
        let deserialized = AvatarData::from_json(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }
}
