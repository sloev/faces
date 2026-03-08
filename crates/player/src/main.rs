use macroquad::prelude::*;
use player::animation::AnimationPlayer;
use shared::AvatarData;

fn window_conf() -> Conf {
    Conf {
        window_title: "Avatar Player - 2.5D Engine".to_owned(),
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
        // Simple subtle depth effect: darker at the bottom
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
            macroquad::file::set_pc_assets_folder(path.to_str().unwrap());
        }
    }

    let json_data = match load_string("assets/timeline.json").await {
        Ok(json) => json,
        Err(_) => {
            println!("Error: assets/timeline.json not found.");
            return;
        }
    };

    let data = AvatarData::from_json(&json_data).expect("Invalid JSON");
    let mut player = AnimationPlayer::new(data.clone());

    let texture = match load_texture("assets/face.png").await {
        Ok(t) => t,
        Err(_) => {
            // Fallback for face.jpg if face.png doesn't exist
            match load_texture("assets/face.jpg").await {
                Ok(t) => t,
                Err(_) => Texture2D::from_rgba8(2, 2, &[200, 200, 200, 255, 220, 220, 220, 255, 180, 180, 180, 255, 200, 200, 200, 255])
            }
        }
    };
    texture.set_filter(FilterMode::Linear);

    let pipeline = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            ..Default::default()
        },
    ).expect("Failed to load shader");

    loop {
        clear_background(Color::new(0.1, 0.1, 0.12, 1.0));

        let dt = get_frame_time();
        player.update(dt);

        let vertices = player.get_current_pose();
        let mq_vertices: Vec<macroquad::models::Vertex> = vertices
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let uv = data.base_uvs[i];
                macroquad::models::Vertex {
                    position: vec3(v.x * 600.0 + 100.0, v.y * 600.0 + 100.0, 0.0),
                    uv: vec2(uv.x, uv.y),
                    color: [255, 255, 255, 255],
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                }
            })
            .collect();

        gl_use_material(&pipeline);
        draw_mesh(&Mesh {
            vertices: mq_vertices,
            indices: data.mesh_indices.iter().map(|&i| i as u16).collect(),
            texture: Some(texture.clone()),
        });
        gl_use_default_material();

        draw_rectangle(10.0, 10.0, 300.0, 100.0, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_text(&format!("State: {:?}", player.state), 20.0, 35.0, 25.0, WHITE);
        draw_text("Controls:", 20.0, 60.0, 20.0, GRAY);
        draw_text("[H] Say 'Hello'  [Space] Random Clip", 20.0, 85.0, 20.0, LIGHTGRAY);

        if is_key_pressed(KeyCode::H) {
            player.transition_to("talk_hello".to_string());
        }
        
        if is_key_pressed(KeyCode::Space) {
            let clips: Vec<String> = data.clips.keys().cloned().collect();
            let idx = macroquad::rand::gen_range(0, clips.len());
            player.transition_to(clips[idx].clone());
        }

        next_frame().await
    }
}
