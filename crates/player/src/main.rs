use macroquad::prelude::*;
use player::animation::AnimationPlayer;
use shared::AvatarData;

fn window_conf() -> Conf {
    Conf {
        window_title: "Faces Engine".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

struct Resources {
    player: AnimationPlayer,
    texture: Texture2D,
}

enum AppState {
    Waiting,
    Loading,
    Error(String),
    Running(Resources),
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState::Loading;
    let mut frame_count = 0;

    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(mut path) = std::env::current_exe() {
            path.pop();
            if path.join("assets").exists() {
                macroquad::file::set_pc_assets_folder(path.to_str().unwrap_or("."));
            }
        }
    }

    loop {
        clear_background(BLACK);
        frame_count += 1;

        match state {
            AppState::Waiting => {
                if frame_count > 30 {
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                let res = async {
                    // Try multiple paths for robustness
                    let json_data = if let Ok(d) = load_string("assets/timeline.json").await {
                        Ok(d)
                    } else {
                        load_string("timeline.json").await
                    }.map_err(|_| "timeline.json missing")?;
                    
                    let data = AvatarData::from_json(&json_data).map_err(|e| format!("JSON error: {:?}", e))?;
                    
                    let texture = if let Ok(t) = load_texture("assets/face.jpg").await {
                        Ok(t)
                    } else {
                        load_texture("face.jpg").await
                    }.map_err(|_| "face.jpg missing")?;
                    
                    texture.set_filter(FilterMode::Linear);
                    
                    Ok(Resources {
                        player: AnimationPlayer::new(data),
                        texture,
                    })
                }.await;

                match res {
                    Ok(r) => {
                        state = AppState::Running(r);
                        println!("RENDER_LOOP_STARTED");
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        state = AppState::Error(e);
                    }
                }
            }
            AppState::Error(ref e) => {
                clear_background(RED);
                // No draw_text here to prevent early panic
            }
            AppState::Running(ref mut res) => {
                clear_background(DARKGRAY);
                res.player.update(get_frame_time());

                let vertices = res.player.get_current_pose();
                if !vertices.is_empty() {
                    let mq_vertices: Vec<macroquad::models::Vertex> = vertices
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let uv = res.player.data.base_uvs.get(i).cloned().unwrap_or(shared::Vertex { x: 0.0, y: 0.0 });
                            macroquad::models::Vertex {
                                position: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                                uv: vec2(uv.x, uv.y),
                                color: [255, 255, 255, 255],
                                normal: vec4(0.0, 0.0, 1.0, 0.0),
                            }
                        })
                        .collect();

                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: res.player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: Some(res.texture.clone()),
                    });
                }
            }
        }

        next_frame().await
    }
}
