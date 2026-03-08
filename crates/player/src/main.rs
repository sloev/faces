use macroquad::prelude::*;
use player::animation::AnimationPlayer;
use shared::AvatarData;

fn window_conf() -> Conf {
    Conf {
        window_title: "Faces Engine - Debug Mode".to_owned(),
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
                if frame_count > 20 {
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                let res = async {
                    let json_data = load_string("assets/timeline.json").await
                        .map_err(|e| format!("JSON load failed: {:?}", e))?;
                    let data = AvatarData::from_json(&json_data)
                        .map_err(|e| format!("JSON parse failed: {:?}", e))?;
                    let texture = load_texture("assets/face.jpg").await
                        .map_err(|e| format!("Texture load failed: {:?}", e))?;
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
                        macroquad::logging::error!("{}", &e);
                        state = AppState::Error(e);
                    }
                }
            }
            AppState::Error(ref e) => {
                clear_background(RED);
                // Non-panicking draw_text
                draw_text(&format!("Error: {}", e), 20.0, 20.0, 20.0, WHITE);
            }
            AppState::Running(ref mut res) => {
                clear_background(DARKGRAY);
                
                let dt = get_frame_time();
                res.player.update(dt);

                let vertices = res.player.get_current_pose();
                if !vertices.is_empty() {
                    // COORDINATE AUDIT LOG (First frame only)
                    static mut LOGGED_COORDS: bool = false;
                    unsafe {
                        if !LOGGED_COORDS {
                            let v = vertices[0];
                            let sx = v.x * 600.0 + 100.0;
                            let sy = v.y * 600.0 + 100.0;
                            macroquad::logging::info!("{}", &format!("DEBUG_COORDS: Raw({:.2}, {:.2}) -> Screen({:.2}, {:.2})", v.x, v.y, sx, sy));
                            LOGGED_COORDS = true;
                        }
                    }

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

                    // USE DEFAULT MATERIAL
                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: res.player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: Some(res.texture.clone()),
                    });

                    // WIREFRAME DEBUG MODE
                    let indices = &res.player.data.mesh_indices;
                    for i in (0..indices.len()).step_by(3) {
                        if i + 2 < indices.len() {
                            let v1 = vertices[indices[i] as usize];
                            let v2 = vertices[indices[i+1] as usize];
                            let v3 = vertices[indices[i+2] as usize];
                            let p1 = vec2(v1.x * 600.0 + 100.0, v1.y * 600.0 + 100.0);
                            let p2 = vec2(v2.x * 600.0 + 100.0, v2.y * 600.0 + 100.0);
                            let p3 = vec2(v3.x * 600.0 + 100.0, v3.y * 600.0 + 100.0);
                            draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, RED);
                            draw_line(p2.x, p2.y, p3.x, p3.y, 1.0, RED);
                            draw_line(p3.x, p3.y, p1.x, p1.y, 1.0, RED);
                        }
                    }
                }

                draw_rectangle(10.0, 10.0, 300.0, 100.0, Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text(&format!("State: {:?}", res.player.state), 20.0, 35.0, 25.0, WHITE);
                draw_text("[H] Hello  [Space] Random", 20.0, 85.0, 20.0, LIGHTGRAY);

                if is_key_pressed(KeyCode::H) {
                    res.player.transition_to("talk_hello_world".to_string());
                }
                if is_key_pressed(KeyCode::Space) {
                    let clips: Vec<String> = res.player.data.clips.keys().cloned().collect();
                    if !clips.is_empty() {
                        let idx = macroquad::rand::gen_range(0, clips.len());
                        res.player.transition_to(clips[idx].clone());
                    }
                }
            }
        }

        next_frame().await
    }
}
