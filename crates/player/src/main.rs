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

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(mut path) = std::env::current_exe() {
            path.pop();
            macroquad::file::set_pc_assets_folder(path.to_str().unwrap_or("."));
        }
    }

    // 1. Load Data
    let json_data = match load_string("assets/timeline.json").await {
        Ok(json) => json,
        Err(_) => return,
    };

    let data = match AvatarData::from_json(&json_data) {
        Ok(d) => d,
        Err(_) => return,
    };

    let mut player = AnimationPlayer::new(data.clone());

    // 2. Load Texture
    let texture_opt = match load_texture("assets/face.jpg").await {
        Ok(t) => {
            t.set_filter(FilterMode::Linear);
            Some(t)
        },
        Err(_) => None,
    };

    // 3. Load Shader
    let pipeline_opt = match load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams { ..Default::default() },
    ) {
        Ok(p) => Some(p),
        Err(_) => None,
    };

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();
        player.update(dt);

        let vertices = player.get_current_pose();
        if !vertices.is_empty() {
            if let (Some(ref texture), Some(ref pipeline)) = (&texture_opt, &pipeline_opt) {
                let mq_vertices: Vec<macroquad::models::Vertex> = vertices
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let uv = data.base_uvs.get(i).cloned().unwrap_or(shared::Vertex { x: 0.0, y: 0.0 });
                        macroquad::models::Vertex {
                            position: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                            uv: vec2(uv.x, uv.y),
                            color: [255, 255, 255, 255],
                            normal: vec4(0.0, 0.0, 1.0, 0.0),
                        }
                    })
                    .collect();

                gl_use_material(pipeline);
                draw_mesh(&Mesh {
                    vertices: mq_vertices,
                    indices: data.mesh_indices.iter().map(|&i| i as u16).collect(),
                    texture: Some(texture.clone()),
                });
                gl_use_default_material();
            }
        }

        if is_key_pressed(KeyCode::H) {
            player.transition_to("talk_hello_world".to_string());
        }
        
        if is_key_pressed(KeyCode::Space) {
            let clips: Vec<String> = data.clips.keys().cloned().collect();
            if !clips.is_empty() {
                let idx = macroquad::rand::gen_range(0, clips.len());
                player.transition_to(clips[idx].clone());
            }
        }

        next_frame().await
    }
}
