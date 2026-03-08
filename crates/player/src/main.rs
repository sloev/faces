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
    Error,
    Running {
        player: AnimationPlayer,
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
                clear_background(BLUE);
                if frame_count > 10 {
                    state = AppState::Loading;
                }
            }
            AppState::Loading => {
                clear_background(YELLOW);
                
                let assets_ready = async {
                    let json_data = load_string("assets/timeline.json").await.ok()?;
                    let data = AvatarData::from_json(&json_data).ok()?;
                    Some(AnimationPlayer::new(data))
                }.await;

                if let Some(player) = assets_ready {
                    state = AppState::Running { player };
                    #[cfg(target_family = "wasm")] 
                    macroquad::logging::info!("RENDER_LOOP_STARTED");
                } else {
                    state = AppState::Error;
                }
            }
            AppState::Error => {
                clear_background(RED);
            }
            AppState::Running { ref mut player } => {
                clear_background(GREEN);
                player.update(get_frame_time());
            }
        }

        next_frame().await
    }
}
