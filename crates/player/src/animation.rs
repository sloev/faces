use shared::{AvatarData, Vertex, Clip, Frame};

/// The various states the animation player can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    /// Looping an idle animation.
    Idle { clip_name: String, time: f32 },
    /// Playing a specific talking animation.
    Talking { clip_name: String, time: f32, queue: Vec<String> },
    /// Smoothly morphing from the last known pose to a new clip's first frame.
    Transitioning { from_pose: Vec<Vertex>, to_clip: String, elapsed: f32, duration: f32 },
}

pub struct AnimationPlayer {
    pub data: AvatarData,
    pub state: PlayerState,
    pub transition_duration: f32,
}

impl AnimationPlayer {
    pub fn new(data: AvatarData) -> Self {
        let first_clip = data.clips.keys().next().cloned().unwrap_or_default();
        Self {
            data,
            state: PlayerState::Idle { clip_name: first_clip, time: 0.0 },
            transition_duration: 0.2,
        }
    }

    pub fn update(&mut self, dt: f32) {
        match &mut self.state {
            PlayerState::Idle { clip_name, time } => {
                *time += dt;
                let clip = self.data.clips.get(clip_name).expect("Clip not found");
                let duration = clip.frames.last().map(|f| f.timestamp).unwrap_or(0.0);
                if *time >= duration {
                    let idles: Vec<&String> = self.data.clips.keys().filter(|k| k.starts_with("idle")).collect();
                    if !idles.is_empty() {
                        let idx = macroquad::rand::gen_range(0, idles.len());
                        self.transition_to(idles[idx].clone());
                    }
                }
            }
            PlayerState::Talking { clip_name, time, queue } => {
                *time += dt;
                let clip = self.data.clips.get(clip_name).expect("Clip not found");
                let duration = clip.frames.last().map(|f| f.timestamp).unwrap_or(0.0);
                if *time >= duration {
                    if let Some(next_clip) = queue.pop() {
                        self.transition_to(next_clip);
                    } else {
                        let idle_clip = self.data.clips.keys().find(|k| k.starts_with("idle")).cloned().unwrap_or_default();
                        self.transition_to(idle_clip);
                    }
                }
            }
            PlayerState::Transitioning { to_clip, elapsed, duration, .. } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    self.state = PlayerState::Idle { clip_name: to_clip.clone(), time: 0.0 };
                }
            }
        }
    }

    pub fn transition_to(&mut self, next_clip: String) {
        if self.data.clips.contains_key(&next_clip) {
            let current_pose = self.get_current_pose();
            self.state = PlayerState::Transitioning { from_pose: current_pose, to_clip: next_clip, elapsed: 0.0, duration: self.transition_duration };
        }
    }

    pub fn get_current_pose(&self) -> Vec<Vertex> {
        match &self.state {
            PlayerState::Idle { clip_name, time } | PlayerState::Talking { clip_name, time, .. } => {
                let clip = self.data.clips.get(clip_name).expect("Clip not found");
                interpolate_clip(clip, *time)
            }
            PlayerState::Transitioning { from_pose, to_clip, elapsed, duration } => {
                let target_clip = self.data.clips.get(to_clip).expect("Clip not found");
                let target_pose = interpolate_clip(target_clip, 0.0);
                let t = (*elapsed / *duration).clamp(0.0, 1.0);
                from_pose.iter().zip(target_pose.iter()).map(|(a, b)| lerp_vertex(*a, *b, t)).collect()
            }
        }
    }
}

pub fn lerp_vertex(a: Vertex, b: Vertex, t: f32) -> Vertex {
    Vertex { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t }
}

pub fn interpolate_clip(clip: &Clip, time: f32) -> Vec<Vertex> {
    if clip.frames.is_empty() { return vec![]; }
    if clip.frames.len() == 1 { return clip.frames[0].vertices.clone(); }
    let mut next_idx = 0;
    while next_idx < clip.frames.len() && clip.frames[next_idx].timestamp < time { next_idx += 1; }
    if next_idx == 0 { return clip.frames[0].vertices.clone(); }
    if next_idx == clip.frames.len() { return clip.frames.last().unwrap().vertices.clone(); }
    let prev_frame = &clip.frames[next_idx - 1];
    let next_frame = &clip.frames[next_idx];
    let t = (time - prev_frame.timestamp) / (next_frame.timestamp - prev_frame.timestamp);
    prev_frame.vertices.iter().zip(next_frame.vertices.iter()).map(|(a, b)| lerp_vertex(*a, *b, t.clamp(0.0, 1.0))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_avatar() -> AvatarData {
        let mut clips = HashMap::new();
        clips.insert("idle_1".to_string(), Clip {
            name: "idle_1".to_string(),
            frames: vec![
                Frame { timestamp: 0.0, vertices: vec![Vertex { x: 0.0, y: 0.0 }] },
                Frame { timestamp: 1.0, vertices: vec![Vertex { x: 1.0, y: 1.0 }] },
            ],
        });
        clips.insert("talk_1".to_string(), Clip {
            name: "talk_1".to_string(),
            frames: vec![Frame { timestamp: 0.0, vertices: vec![Vertex { x: 0.5, y: 0.5 }] }],
        });
        AvatarData { clips, mesh_indices: vec![], base_uvs: vec![] }
    }

    #[test]
    fn test_lerp_logic() {
        let v1 = Vertex { x: 0.0, y: 0.0 };
        let v2 = Vertex { x: 10.0, y: 10.0 };
        assert_eq!(lerp_vertex(v1, v2, 0.5).x, 5.0);
    }

    #[test]
    fn test_fsm_transitions() {
        let mut player = AnimationPlayer::new(mock_avatar());
        
        // Trigger transition
        player.transition_to("talk_1".to_string());
        if let PlayerState::Transitioning { to_clip, .. } = &player.state {
            assert_eq!(to_clip, "talk_1");
        } else {
            panic!("Should be transitioning");
        }

        // Complete transition (duration is 0.2)
        player.update(0.3); 
        if let PlayerState::Idle { clip_name, .. } = &player.state {
            assert_eq!(clip_name, "talk_1");
        } else {
            panic!("Should have finished transition, state is {:?}", player.state);
        }
    }

    #[test]
    fn test_interpolation_between_frames() {
        let data = mock_avatar();
        let clip = data.clips.get("idle_1").unwrap();
        let pose = interpolate_clip(clip, 0.5);
        assert_eq!(pose[0].x, 0.5);
    }
}
