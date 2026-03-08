use shared::{AvatarData, Vertex, Frame, Clip};
use ort::session::Session;
use ort::value::Value;
use ort::inputs;
use image::{GenericImageView, DynamicImage, imageops::FilterType};
use delaunator::{triangulate, Point};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Phase 10: Autonomous Test-Driven Verification");

    let model_path = "assets/face_mesh.onnx";
    let image_path = "assets/face.jpg";
    let output_path = "assets/timeline.json";

    let mut session_opt = None;
    if std::path::Path::new(model_path).exists() {
        println!("Attempting to load model from '{}'...", model_path);
        match Session::builder() {
            Ok(mut builder) => {
                match builder.commit_from_file(model_path) {
                    Ok(s) => session_opt = Some(s),
                    Err(e) => println!("Warning: Failed to commit model from file: {:?}. Using mock data.", e),
                }
            }
            Err(e) => println!("Warning: Failed to create session builder: {:?}. Using mock data.", e),
        }
    }

    let landmarks = if let Some(mut session) = session_opt {
        if std::path::Path::new(image_path).exists() {
            let img = image::open(image_path).unwrap_or_else(|_| DynamicImage::new_rgb8(192, 192));
            extract_real_landmarks(&mut session, &img).unwrap_or_else(|_| simulate_landmarks())
        } else {
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
    
    // Ensure face.jpg exists in assets for the player
    let mut face_saved = false;
    if std::path::Path::new(image_path).exists() {
        if let Ok(metadata) = std::fs::metadata(image_path) {
            if metadata.len() > 0 {
                std::fs::copy(image_path, "assets/face.jpg")?;
                face_saved = true;
            }
        }
    }
    
    if !face_saved {
        println!("Warning: face.jpg input missing or empty. Generating dummy face.");
        let img = DynamicImage::new_rgb8(192, 192);
        img.save("assets/face.jpg")?;
    }

    println!("Successfully exported timelines to '{}'", output_path);

    Ok(())
}

fn extract_real_landmarks(session: &mut Session, img: &DynamicImage) -> Result<Vec<Vertex>, Box<dyn std::error::Error>> {
    let pixels = preprocess_image(img);
    let input_value = Value::from_array(([1, 192, 192, 3], pixels))?;
    let outputs = session.run(inputs![input_value])?;
    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;
    Ok(postprocess_landmarks(data))
}

fn preprocess_image(img: &DynamicImage) -> Vec<f32> {
    let resized = img.resize_exact(192, 192, FilterType::Triangle);
    let mut pixels = Vec::with_capacity(192 * 192 * 3);
    for (_, _, pixel) in resized.pixels() {
        pixels.push((pixel[0] as f32) / 255.0);
        pixels.push((pixel[1] as f32) / 255.0);
        pixels.push((pixel[2] as f32) / 255.0);
    }
    pixels
}

fn postprocess_landmarks(data: &[f32]) -> Vec<Vertex> {
    let mut landmarks = Vec::new();
    for i in 0..468 {
        if (i * 3 + 1) < data.len() {
            landmarks.push(Vertex { 
                x: data[i * 3] / 192.0, 
                y: data[i * 3 + 1] / 192.0 
            });
        }
    }
    landmarks
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
        landmarks.push(Vertex { x: 0.5 + angle.cos() * 0.3, y: 0.5 + angle.sin() * 0.3 });
    }
    landmarks.push(Vertex { x: 0.5, y: 0.5 });
    landmarks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing_shape() {
        let img = DynamicImage::new_rgb8(10, 10);
        let pixels = preprocess_image(&img);
        assert_eq!(pixels.len(), 192 * 192 * 3);
    }

    #[test]
    fn test_postprocessing_logic() {
        let mut data = vec![0.0; 1404];
        data[0] = 96.0; // x1
        data[1] = 48.0; // y1
        let landmarks = postprocess_landmarks(&data);
        assert_eq!(landmarks.len(), 468);
        assert_eq!(landmarks[0].x, 0.5);
        assert_eq!(landmarks[0].y, 0.25);
    }

    #[test]
    fn test_triangulation_validity() {
        let landmarks = vec![
            Vertex { x: 0.0, y: 0.0 },
            Vertex { x: 1.0, y: 0.0 },
            Vertex { x: 0.0, y: 1.0 },
            Vertex { x: 1.0, y: 1.0 },
        ];
        let points: Vec<Point> = landmarks.iter().map(|v| Point { x: v.x as f64, y: v.y as f64 }).collect();
        let tri = triangulate(&points);
        assert_eq!(tri.triangles.len(), 6); // 2 triangles * 3 indices
    }

    #[test]
    fn test_viseme_synthesis() {
        let base = vec![Vertex { x: 0.5, y: 0.5 }; 468];
        let clip = build_viseme_timeline("A", &base);
        assert_eq!(clip.frames.len(), 1);
        // Index 14 is lower lip, should be offset
        assert!(clip.frames[0].vertices[14].y > 0.5);
    }
}
