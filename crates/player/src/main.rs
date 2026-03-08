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

enum AppState {
    Waiting,
    Loading,
    Error(String),
    Running {
        player: AnimationPlayer,
        texture: Texture2D,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState::Waiting;
    let mut frame_count = 0;

    loop {
        clear_background(BLACK);
        frame_count += 1;

        match state {
            AppState::Waiting => {
                if frame_count > 200 { // 3 second stabilization
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                let res = async {
                    let json_data = if let Ok(d) = load_string("assets/timeline.json").await {
                        d
                    } else {
                        load_string("timeline.json").await
                            .map_err(|_| "timeline.json missing")?
                    };
                    
                    let data = AvatarData::from_json(&json_data).map_err(|e| format!("JSON error: {:?}", e))?;
                    
                    // Use load_image for safer Wasm decoding
                    let img = if let Ok(i) = load_image("assets/face.jpg").await {
                        i
                    } else {
                        load_image("face.jpg").await
                            .map_err(|_| "face.jpg missing")?
                    };
                    
                    let texture = Texture2D::from_image(&img);
                    texture.set_filter(FilterMode::Linear);
                    
                    Ok((AnimationPlayer::new(data), texture))
                }.await;

                match res {
                    Ok((player, texture)) => {
                        state = AppState::Running { player, texture };
                        println!("RENDER_LOOP_STARTED");
                    }
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        state = AppState::Error(e);
                    }
                }
            }
            AppState::Error(_) => {
                clear_background(RED);
            }
            AppState::Running { ref mut player, ref texture } => {
                clear_background(DARKGRAY);
                player.update(get_frame_time());

                let vertices = player.get_current_pose();
                if !vertices.is_empty() {
                    let mq_vertices: Vec<macroquad::models::Vertex> = vertices
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let uv = player.data.base_uvs.get(i).cloned().unwrap_or(shared::Vertex { x: 0.0, y: 0.0 });
                            macroquad::models::Vertex {
                                position: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                                uv: vec2(uv.x, uv.y),
                                color: WHITE.into(),
                                normal: vec4(0.0, 0.0, 1.0, 0.0),
                            }
                        })
                        .collect();

                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: Some(texture.clone()),
                    });
                }
            }
        }

        next_frame().await
    }
}
