# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based graphics application using the Vulkan API through the Vulkano library to render procedurally generated circles in fullscreen mode. The project demonstrates advanced vertex shader programming with procedural geometry generation and real-time animation.

## Build and Development Commands

```bash
cargo check           # Compile check
cargo build           # Debug build
cargo run             # Run the procedural circle application
cargo build --release # Release build
```

## Architecture

**Core Technologies:**
- Rust (edition 2024)
- Vulkan API via Vulkano 0.35
- Winit 0.30 for windowing
- GLSL shaders compiled at build time

**Module Structure:**
- `main.rs` - Event loop and application lifecycle using winit's ApplicationHandler
- `render_context.rs` - Vulkan device/instance setup and resource allocation
- `window_context.rs` - Swapchain management, rendering pipeline, and frame rendering
- `control.rs` - Input state management for rotation controls
- `model.rs` - Circle generation parameters (radius, segments, etc.)
- `vert.glsl`/`frag.glsl` - Vertex and fragment shaders with procedural circle generation

**Rendering Pipeline:**
- Uses Vulkan's low-level graphics API through Vulkano
- Procedural vertex generation in vertex shader (no vertex buffers)
- Line-based primitive rendering (PrimitiveTopology::LineList) 
- Push constants for dynamic data (rotation angles, aspect ratio, circle parameters)
- Proper swapchain recreation for window resizing
- RAII pattern for Vulkan resource management

## Application Behavior

The application renders procedurally generated circles that rotate in real-time. All geometry is generated mathematically in the vertex shader using trigonometric functions. Controls:
- Arrow keys: Control rotation of circles
- Escape: Exit application  
- Runs in fullscreen borderless window mode

**Key Features:**
- No vertex data - everything generated procedurally in shaders
- Real-time rotation animation
- Aspect ratio correction for perfect circles on any screen
- Configurable circle parameters (radius, segments)

## Development Patterns

**Error Handling:** Currently uses `unwrap()` extensively (development-focused, not production-ready)

**Shader Integration:** GLSL shaders are embedded and compiled at build time via vulkano-shaders

**Resource Management:** Leverages Rust's ownership system for safe Vulkan resource handling

**Procedural Generation:** All vertices generated mathematically using `gl_VertexIndex`, trigonometric functions, and push constants

**Geometric Transformations:** Vertex shader handles real-time circle rotation and aspect ratio correction

## Testing the Application

Run `cargo run` and test with arrow keys for rotation. Visual output verification is the primary testing method as this is a graphics application. The circles should appear as perfect circles (not ovals) and rotate smoothly.