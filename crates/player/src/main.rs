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
    let mut state = AppState::Waiting;
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
                if frame_count > 60 { // Wait 1 second (at 60fps)
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                let res = async {
                    // Try different paths for Wasm flexibility
                    let paths = ["assets/timeline.json", "timeline.json"];
                    let mut json_data = None;
                    for p in paths {
                        if let Ok(data) = load_string(p).await {
                            json_data = Some(data);
                            break;
                        }
                    }
                    
                    let json_data = json_data.ok_or("timeline.json not found in assets/ or root")?;
                    let data = AvatarData::from_json(&json_data).map_err(|e| format!("Parse error: {:?}", e))?;
                    
                    let texture = load_texture("assets/face.jpg").await
                        .or_else(|_| load_texture("face.jpg").await)
                        .map_err(|e| format!("Texture error: {:?}", e))?;
                    
                    texture.set_filter(FilterMode::Linear);
                    
                    Ok(Resources {
                        player: AnimationPlayer::new(data),
                        texture,
                    })
                }.await;

                match res {
                    Ok(r) => {
                        state = AppState::Running(r);
                        #[cfg(target_family = "wasm")]
                        macroquad::logging::info!("RENDER_LOOP_STARTED");
                    }
                    Err(e) => {
                        #[cfg(target_family = "wasm")]
                        macroquad::logging::error(&e);
                        state = AppState::Error(e);
                    }
                }
            }
            AppState::Error(_) => {
                clear_background(RED);
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
