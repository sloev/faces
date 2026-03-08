use shared::{AvatarData, Vertex, Frame, Clip};
use ort::session::Session;
use ort::value::Value;
use ort::inputs;
use image::{GenericImageView, DynamicImage, imageops::FilterType};
use delaunator::{triangulate, Point};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Phase 8: CI/CD Pipeline and Asset Bundling");

    let model_path = "assets/face_mesh.onnx";
    let image_path = "assets/face.jpg";
    let output_path = "assets/timeline.json";

    let mut session_opt = None;
    if std::path::Path::new(model_path).exists() {
        println!("Attempting to load model from '{}'...", model_path);
        match Session::builder() {
            Ok(builder) => {
                match builder.commit_from_file(model_path) {
                    Ok(s) => session_opt = Some(s),
                    Err(e) => println!("Warning: Failed to commit model from file: {:?}. Using mock data.", e),
                }
            }
            Err(e) => println!("Warning: Failed to create session builder: {:?}. Using mock data.", e),
        }
    } else {
        println!("Warning: '{}' not found. Using mock ML data.", model_path);
    }

    let landmarks = if let Some(mut session) = session_opt {
        if std::path::Path::new(image_path).exists() {
            println!("Processing image '{}'...", image_path);
            match image::open(image_path) {
                Ok(img) => {
                    match extract_real_landmarks(&mut session, &img) {
                        Ok(l) => l,
                        Err(e) => {
                            println!("Warning: Extraction failed: {:?}. Using mock landmarks.", e);
                            simulate_landmarks()
                        }
                    }
                }
                Err(e) => {
                    println!("Warning: Failed to open image: {:?}. Using mock landmarks.", e);
                    simulate_landmarks()
                }
            }
        } else {
            println!("Warning: '{}' not found. Using mock ML data.", image_path);
            simulate_landmarks()
        }
    } else {
        simulate_landmarks()
    };

    let points: Vec<Point> = landmarks.iter()
        .map(|v| Point { x: v.x as f64, y: v.y as f64 })
        .collect();
    let tri = triangulate(&points);
    let indices: Vec<u32> = tri.triangles.iter().map(|&i| i as u32).collect();

    let mut clips = HashMap::new();
    clips.insert("idle_base".to_string(), Clip {
        name: "idle_base".to_string(),
        frames: vec![Frame { timestamp: 0.0, vertices: landmarks.clone() }],
    });

    let sentences = vec!["Hello World", "Rust is awesome"];
    for text in sentences {
        let clip_name = format!("talk_{}", text.to_lowercase().replace(" ", "_"));
        clips.insert(clip_name, build_viseme_timeline(text, &landmarks));
    }

    let data = AvatarData {
        clips,
        mesh_indices: indices,
        base_uvs: landmarks, 
    };

    std::fs::create_dir_all("assets")?;
    std::fs::write(output_path, data.to_json()?)?;
    println!("Successfully exported timelines to '{}'", output_path);

    Ok(())
}

fn extract_real_landmarks(session: &mut Session, img: &DynamicImage) -> Result<Vec<Vertex>, Box<dyn std::error::Error>> {
    let resized = img.resize_exact(192, 192, FilterType::Triangle);
    let mut pixels = Vec::with_capacity(192 * 192 * 3);

    for (_, _, pixel) in resized.pixels() {
        pixels.push((pixel[0] as f32) / 255.0);
        pixels.push((pixel[1] as f32) / 255.0);
        pixels.push((pixel[2] as f32) / 255.0);
    }

    let input_value = Value::from_array(([1, 192, 192, 3], pixels))?;
    let outputs = session.run(inputs![input_value])?;
    
    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;
    
    let mut landmarks = Vec::new();
    for i in 0..468 {
        if (i * 3 + 1) < data.len() {
            landmarks.push(Vertex { 
                x: data[i * 3] / 192.0, 
                y: data[i * 3 + 1] / 192.0 
            });
        }
    }

    Ok(landmarks)
}

fn build_viseme_timeline(text: &str, base_pose: &[Vertex]) -> Clip {
    let mut frames = Vec::new();
    let mut current_time = 0.0;
    let duration_per_char = 0.12;

    for c in text.to_lowercase().chars() {
        let mut next_pose = base_pose.to_vec();
        let offset = match c {
            'a' | 'e' | 'i' => 0.06,
            'o' | 'u' | 'w' => 0.04,
            'f' | 'v' => 0.015,
            _ => 0.0,
        };

        if offset > 0.0 {
            let lower_lip = [14, 15, 17, 87, 88, 317, 318];
            for &idx in &lower_lip {
                if idx < next_pose.len() {
                    next_pose[idx].y += offset;
                }
            }
        }
        frames.push(Frame { timestamp: current_time, vertices: next_pose });
        current_time += duration_per_char;
    }
    Clip { name: text.to_string(), frames }
}

fn simulate_landmarks() -> Vec<Vertex> {
    let mut landmarks = Vec::new();
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * std::f32::consts::PI * 2.0;
        landmarks.push(Vertex { 
            x: 0.5 + angle.cos() * 0.3, 
            y: 0.5 + angle.sin() * 0.3 
        });
    }
    landmarks.push(Vertex { x: 0.5, y: 0.5 });
    landmarks
}
