# Faces: Audio-Driven 2D/2.5D Avatar Engine

A lightweight, cross-platform facial animation engine built entirely in Rust. `Faces` uses machine learning to extract facial landmarks, performs Delaunay triangulation to create a dynamic 2.5D mesh, and synthesizes text-to-viseme timelines for real-time playback.

## 🚀 Live Demo
**[View the WebAssembly Demo on GitHub Pages](https://sloev.github.io/faces/)**

---

## 🛠 Tech Stack

*   **Language:** 100% Idiomatic Rust
*   **Graphics:** [Macroquad](https://macroquad.rs/) (High-level, cross-platform 2D/3D)
*   **Machine Learning:** [ORT](https://github.com/pykeio/ort) (ONNX Runtime) for MediaPipe Face Mesh inference
*   **Math:** [Delaunator](https://github.com/fogleman/delaunator) for robust 2D triangulation
*   **Serialization:** [Serde](https://serde.rs/) & `serde_json` for the animation contract

---

## 📂 Project Structure

The project is organized as a Cargo workspace:

*   **`crates/shared`**: Defines the data contract (Vertices, Frames, Clips, AvatarData) used by both the generator and player.
*   **`crates/generator`**: The offline ML pipeline.
    *   Downloads and loads the MediaPipe Face Mesh ONNX model.
    *   Processes input images (`face.jpg`) to extract 468 landmarks.
    *   Generates Delaunay triangulation indices.
    *   Synthesizes character-to-viseme timelines from text strings.
    *   Exports everything to `assets/timeline.json`.
*   **`crates/player`**: The lightweight rendering engine.
    *   Implements a Finite State Machine (Idle, Talking, Transitioning).
    *   Performs real-time vertex interpolation (LERP) between animation frames.
    *   Uses custom GLSL shaders for 2.5D depth and lighting effects.
    *   Compiles to Native (Linux/Windows/macOS/ARM) and WebAssembly.

---

## ⚙️ Automated CI/CD Pipeline

This repository is configured with a zero-setup GitHub Actions workflow (`.github/workflows/release.yml`):

1.  **Asset Synthesis:** On every push, the CI runner downloads the latest MediaPipe model, runs the `generator` to build the mesh, and synthesizes the animation data.
2.  **Multi-Platform Bundling:**
    *   **Linux (x86_64):** Generates a standalone, zero-dependency **AppImage**.
    *   **Raspberry Pi (AArch64):** Cross-compiles and packages a `.tar.gz` bundle.
    *   **Web (Wasm):** Compiles to WebAssembly and deploys to **GitHub Pages**.
3.  **Automatic Releases:** Tagging a commit (e.g., `v1.0.0`) automatically creates a GitHub Release with all three platform bundles attached.

---

## 🎮 Controls (Player)

*   **`H`**: Trigger "Hello World" talking sequence.
*   **`Space`**: Play a random animation clip from the synthesized library.
*   **`ESC`**: Exit (Native only).

---

## 🛠 Local Development

### 1. Prerequisites
Ensure you have the Rust toolchain installed. For Linux, you may need system dependencies:
```bash
sudo apt-get install libasound2-dev libudev-dev pkg-config libx11-dev libxi-dev libgl1-mesa-dev
```

### 2. Generate Data
The generator requires `face_mesh.onnx` and `face.jpg` in the `assets/` folder.
```bash
mkdir -p assets
# (Download assets or run the CI pipeline locally)
cargo run -p generator
```

### 3. Run Player
```bash
cargo run -p player
```

### 4. Build for Web
```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown -p player
# Serve the 'static' or 'web-dist' folder
```
