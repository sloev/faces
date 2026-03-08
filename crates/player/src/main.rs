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

const FRAGMENT_SHADER: &str = r#"#version 100
    precision lowp float;
    varying vec2 uv;
    varying vec4 color;
    uniform sampler2D Texture;
    void main() {
        vec4 res = texture2D(Texture, uv) * color;
        res.rgb *= (1.0 - (uv.y * 0.2));
        gl_FragColor = res;
    }
"#;

const VERTEX_SHADER: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying vec2 uv;
    varying vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1.0);
        uv = texcoord;
        color = color0;
    }
"#;

struct Resources {
    player: AnimationPlayer,
    texture: Texture2D,
    pipeline: Material,
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
                    // Strictly relative paths for Wasm compatibility
                    let json_data = load_string("assets/timeline.json").await
                        .map_err(|e| format!("Load failed assets/timeline.json: {:?}", e))?;
                    
                    let data = AvatarData::from_json(&json_data)
                        .map_err(|e| format!("Parse failed timeline.json: {:?}", e))?;
                    
                    let texture = load_texture("assets/face.jpg").await
                        .map_err(|e| format!("Load failed assets/face.jpg: {:?}", e))?;
                    texture.set_filter(FilterMode::Linear);

                    let pipeline = load_material(
                        ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER },
                        MaterialParams { ..Default::default() }
                    ).map_err(|e| format!("Shader error: {:?}", e))?;
                    
                    Ok(Resources {
                        player: AnimationPlayer::new(data),
                        texture,
                        pipeline,
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
                // In Wasm, we rely on the console log above
                #[cfg(not(target_family = "wasm"))]
                draw_text(&format!("Error: {}", e), 20.0, 20.0, 20.0, WHITE);
            }
            AppState::Running(ref mut res) => {
                clear_background(Color::new(0.1, 0.1, 0.12, 1.0));
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
                                color: WHITE.into(),
                                normal: vec4(0.0, 0.0, 1.0, 0.0),
                            }
                        })
                        .collect();

                    gl_use_material(&res.pipeline);
                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: res.player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: Some(res.texture.clone()),
                    });
                    gl_use_default_material();
                }

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
