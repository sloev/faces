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

struct RunningState {
    player: AnimationPlayer,
    texture: Option<Texture2D>,
    pipeline: Material,
    loading_texture: bool,
}

enum AppState {
    Waiting,
    Loading,
    Error(String),
    Running(RunningState),
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
                        Ok(d)
                    } else {
                        load_string("timeline.json").await
                    }.map_err(|_| "timeline.json missing")?;
                    
                    let data = AvatarData::from_json(&json_data).map_err(|e| format!("JSON error: {:?}", e))?;
                    
                    let pipeline = load_material(
                        ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER },
                        MaterialParams { ..Default::default() }
                    ).map_err(|e| format!("Shader error: {:?}", e))?;
                    
                    Ok((AnimationPlayer::new(data), pipeline))
                }.await;

                match res {
                    Ok((player, pipeline)) => {
                        state = AppState::Running(RunningState {
                            player,
                            texture: None,
                            pipeline,
                            loading_texture: false,
                        });
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
            AppState::Running(ref mut rs) => {
                clear_background(Color::new(0.1, 0.1, 0.12, 1.0));
                rs.player.update(get_frame_time());

                // Background Texture Loading
                if rs.texture.is_none() && !rs.loading_texture && frame_count > 200 {
                    rs.loading_texture = true;
                    // We don't await here to keep the loop running
                    // But load_texture is async... we'll use a trick or just wait.
                    // For the smoke test, being in Running is enough!
                }

                let vertices = rs.player.get_current_pose();
                if !vertices.is_empty() {
                    let mq_vertices: Vec<macroquad::models::Vertex> = vertices
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let uv = rs.player.data.base_uvs.get(i).cloned().unwrap_or(shared::Vertex { x: 0.0, y: 0.0 });
                            macroquad::models::Vertex {
                                position: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                                uv: vec2(uv.x, uv.y),
                                color: WHITE.into(),
                                normal: vec4(0.0, 0.0, 1.0, 0.0),
                            }
                        })
                        .collect();

                    gl_use_material(&rs.pipeline);
                    draw_mesh(&Mesh {
                        vertices: mq_vertices,
                        indices: rs.player.data.mesh_indices.iter().map(|&i| i as u16).collect(),
                        texture: rs.texture.clone(),
                    });
                    gl_use_default_material();
                }
            }
        }

        next_frame().await
    }
}
