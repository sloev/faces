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

    if !std::path::Path::new(model_path).exists() {
        println!("Error: '{}' not found. Generator requires this file to run.", model_path);
        return Ok(());
    }

    let mut session = Session::builder()?
        .commit_from_file(model_path)?;

    if !std::path::Path::new(image_path).exists() {
        println!("Error: 'face.jpg' not found. Please run the curl command provided.");
        return Ok(());
    }
    
    let img = image::open(image_path)?;
    let landmarks = extract_real_landmarks(&mut session, &img)?;

    // Delaunay triangulation on the 468 points
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

    // Generate some talking timelines based on these real points
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

    std::fs::write(output_path, data.to_json()?)?;
    println!("Successfully exported REAL ML landmarks and timelines to '{}'", output_path);

    Ok(())
}

fn extract_real_landmarks(session: &mut Session, img: &DynamicImage) -> Result<Vec<Vertex>, Box<dyn std::error::Error>> {
    // 1. Preprocessing: Resize to 192x192 and convert to RGB f32 [0, 1]
    let resized = img.resize_exact(192, 192, FilterType::Triangle);
    let mut pixels = Vec::with_capacity(192 * 192 * 3);

    for (_, _, pixel) in resized.pixels() {
        // MediaPipe Face Mesh expects [0.0, 1.0] normalization
        pixels.push((pixel[0] as f32) / 255.0);
        pixels.push((pixel[1] as f32) / 255.0);
        pixels.push((pixel[2] as f32) / 255.0);
    }

    // 2. Inference: NHWC [1, 192, 192, 3]
    let input_value = Value::from_array(([1, 192, 192, 3], pixels))?;
    let outputs = session.run(inputs![input_value])?;
    
    // 3. Postprocessing: The output is usually the first tensor, 1404 floats (468 * 3)
    let (_, data) = outputs[0].try_extract_tensor::<f32>()?;
    
    let mut landmarks = Vec::new();
    for i in 0..468 {
        // The output coordinates are in pixels (0.0 to 192.0). 
        // We normalize them back to [0.0, 1.0] for our Player.
        let x = data[i * 3] / 192.0;
        let y = data[i * 3 + 1] / 192.0;
        landmarks.push(Vertex { x, y });
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
            // Lower lip indices for MediaPipe Face Mesh
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
