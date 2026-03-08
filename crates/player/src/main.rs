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
    Error,
    Running {
        player: AnimationPlayer,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState::Loading;

    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(mut path) = std::env::current_exe() {
            path.pop();
            if path.join("assets").exists() {
                macroquad::file::set_pc_assets_folder(path.to_str().unwrap_or("."));
            }
        }
    }

    let load_result: Result<AnimationPlayer, String> = async {
        let json_data = load_string("assets/timeline.json").await
            .map_err(|e| {
                let err = format!("Load failed: {:?}", e);
                #[cfg(target_family = "wasm")] macroquad::logging::error(&err);
                err
            })?;
        
        let data = AvatarData::from_json(&json_data)
            .map_err(|e| {
                let err = format!("Parse failed: {:?}", e);
                #[cfg(target_family = "wasm")] macroquad::logging::error(&err);
                err
            })?;
        
        Ok(AnimationPlayer::new(data))
    }.await;

    match load_result {
        Ok(player) => {
            state = AppState::Running { player };
        }
        Err(_) => {
            state = AppState::Error;
        }
    }

    loop {
        match state {
            AppState::Loading => clear_background(BLUE),
            AppState::Error => clear_background(RED),
            AppState::Running { ref mut player } => {
                clear_background(GREEN);
                player.update(get_frame_time());
            }
        }
        next_frame().await
    }
}
