use shared::{AvatarData, Vertex, Clip};

/// The various states the animation player can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    /// Looping an idle animation.
    Idle {
        clip_name: String,
        time: f32,
    },
    /// Playing a specific talking animation.
    Talking {
        clip_name: String,
        time: f32,
        queue: Vec<String>,
    },
    /// Smoothly morphing from the last known pose to a new clip's first frame.
    Transitioning {
        from_pose: Vec<Vertex>,
        to_clip: String,
        elapsed: f32,
        duration: f32,
    },
}

/// Manages the animation state machine and interpolation logic.
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
            state: PlayerState::Idle {
                clip_name: first_clip,
                time: 0.0,
            },
            transition_duration: 0.2,
        }
    }

    pub fn update(&mut self, dt: f32) {
        match &mut self.state {
            PlayerState::Idle { clip_name, time } => {
                *time += dt;
                let clip = self.data.clips.get(clip_name).unwrap();
                let duration = clip.frames.last().map(|f| f.timestamp).unwrap_or(0.0);
                
                if *time >= duration {
                    let idles: Vec<&String> = self.data.clips.keys()
                        .filter(|k| k.starts_with("idle"))
                        .collect();
                    let idx = macroquad::rand::gen_range(0, idles.len());
                    let next_idle = idles[idx].clone();
                    self.transition_to(next_idle);
                }
            }
            PlayerState::Talking { clip_name, time, queue } => {
                *time += dt;
                let clip = self.data.clips.get(clip_name).unwrap();
                let duration = clip.frames.last().map(|f| f.timestamp).unwrap_or(0.0);
                
                if *time >= duration {
                    if let Some(next_clip) = queue.pop() {
                        self.transition_to(next_clip);
                    } else {
                        let idle_clip = self.data.clips.keys()
                            .find(|k| k.starts_with("idle"))
                            .cloned()
                            .unwrap_or_default();
                        self.transition_to(idle_clip);
                    }
                }
            }
            PlayerState::Transitioning { to_clip, elapsed, duration, .. } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    self.state = PlayerState::Idle {
                        clip_name: to_clip.clone(),
                        time: 0.0,
                    };
                }
            }
        }
    }

    pub fn transition_to(&mut self, next_clip: String) {
        if self.data.clips.contains_key(&next_clip) {
            let current_pose = self.get_current_pose();
            self.state = PlayerState::Transitioning {
                from_pose: current_pose,
                to_clip: next_clip,
                elapsed: 0.0,
                duration: self.transition_duration,
            };
        }
    }

    pub fn get_current_pose(&self) -> Vec<Vertex> {
        match &self.state {
            PlayerState::Idle { clip_name, time } | PlayerState::Talking { clip_name, time, .. } => {
                let clip = self.data.clips.get(clip_name).unwrap();
                interpolate_clip(clip, *time)
            }
            PlayerState::Transitioning { from_pose, to_clip, elapsed, duration } => {
                let target_clip = self.data.clips.get(to_clip).unwrap();
                let target_pose = interpolate_clip(target_clip, 0.0);
                let t = (*elapsed / *duration).clamp(0.0, 1.0);
                
                from_pose.iter().zip(target_pose.iter())
                    .map(|(a, b)| lerp_vertex(*a, *b, t))
                    .collect()
            }
        }
    }
}

pub fn lerp_vertex(a: Vertex, b: Vertex, t: f32) -> Vertex {
    Vertex {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

pub fn interpolate_clip(clip: &Clip, time: f32) -> Vec<Vertex> {
    if clip.frames.is_empty() { return vec![]; }
    if clip.frames.len() == 1 { return clip.frames[0].vertices.clone(); }

    let mut next_idx = 0;
    while next_idx < clip.frames.len() && clip.frames[next_idx].timestamp < time {
        next_idx += 1;
    }

    if next_idx == 0 { return clip.frames[0].vertices.clone(); }
    if next_idx == clip.frames.len() { return clip.frames.last().unwrap().vertices.clone(); }

    let prev_frame = &clip.frames[next_idx - 1];
    let next_frame = &clip.frames[next_idx];

    let t = (time - prev_frame.timestamp) / (next_frame.timestamp - prev_frame.timestamp);
    let t = t.clamp(0.0, 1.0);

    prev_frame.vertices.iter().zip(next_frame.vertices.iter())
        .map(|(a, b)| lerp_vertex(*a, *b, t))
        .collect()
}
