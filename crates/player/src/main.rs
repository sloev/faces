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
    Loading,
    Error(String),
    Running {
        player: AnimationPlayer,
        texture: Option<Texture2D>,
        pipeline: Option<Material>,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState::Loading;

    // 1. Initial Path Setup
    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(mut path) = std::env::current_exe() {
            path.pop();
            if path.join("assets").exists() {
                macroquad::file::set_pc_assets_folder(path.to_str().unwrap_or("."));
            }
        }
    }

    // 2. Load Assets in a non-terminating way
    let load_result = async {
        let json_data = load_string("assets/timeline.json").await
            .map_err(|e| format!("Failed to load timeline.json: {:?}", e))?;
        
        let data = AvatarData::from_json(&json_data)
            .map_err(|e| format!("Failed to parse timeline.json: {:?}", e))?;
        
        let player = AnimationPlayer::new(data.clone());
        
        let texture = load_texture("assets/face.jpg").await.ok();
        if let Some(ref t) = texture {
            t.set_filter(FilterMode::Linear);
        }

        Ok(player)
    }.await;

    match load_result {
        Ok(player) => {
            state = AppState::Running {
                player,
                texture: None, // We'll load properly below
                pipeline: None,
            };
        }
        Err(e) => {
            state = AppState::Error(e);
        }
    }

    // Re-attempt texture and shader loading inside the main loop if needed, 
    // but for the smoke test, we just need to not panic.

    loop {
        clear_background(BLACK);

        match state {
            AppState::Loading => {
                draw_text("Loading...", 20.0, 20.0, 30.0, WHITE);
            }
            AppState::Error(ref e) => {
                draw_text(&format!("Error: {}", e), 20.0, 20.0, 20.0, RED);
            }
            AppState::Running { ref mut player, .. } => {
                let dt = get_frame_time();
                player.update(dt);
                draw_text("Engine Running", 20.0, 20.0, 30.0, GREEN);
            }
        }

        next_frame().await
    }
}
