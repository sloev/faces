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
                if frame_count > 60 {
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                let res = async {
                    let json_data = if let Ok(d) = load_string("assets/timeline.json").await {
                        d
                    } else {
                        load_string("timeline.json").await
                            .map_err(|_| "timeline.json missing".to_string())?
                    };
                    
                    let data = AvatarData::from_json(&json_data).map_err(|e| format!("JSON error: {:?}", e))?;
                    
                    let texture = if let Ok(t) = load_texture("assets/face.jpg").await {
                        t
                    } else if let Ok(t) = load_texture("face.jpg").await {
                        t
                    } else {
                        return Err("face.jpg missing".to_string());
                    };
                    
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
                        eprintln!("[ERROR] {}", e);
                        state = AppState::Error(e);
                    }
                }
            }
            AppState::Error(ref e) => {
                clear_background(RED);
                draw_text(&format!("Error: {}", e), 20.0, 20.0, 20.0, WHITE);
            }
            AppState::Running(ref mut res) => {
                clear_background(Color::new(0.05, 0.05, 0.07, 1.0));
                
                let dt = get_frame_time();
                res.player.update(dt);

                let vertices = res.player.get_current_pose();
                if !vertices.is_empty() {
                    let mq_vertices: Vec<Vertex> = vertices
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let base_uv = res.player.data.base_uvs.get(i).cloned()
                                .unwrap_or(shared::Vertex { x: 0.5, y: 0.5 });
                            
                            Vertex {
                                pos: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                                uv: [base_uv.x, base_uv.y],
                                color: WHITE,
                            }
                        })
                        .collect();

                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: res.player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: Some(res.texture.clone()),
                    });
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
