# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based 3D graphics application using the Vulkan API through the Vulkano library to render procedurally generated 3D geometry in fullscreen mode. The project demonstrates vertex shader programming with procedural geometry generation and real-time 3D transformations.

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
- `control.rs` - Input state management for 3D rotation controls
- `model.rs` - 3D geometry parameters (radius, segments, rings, etc.)
- `vert.glsl`/`frag.glsl` - Vertex and fragment shaders with procedural 3D geometry generation

**Rendering Pipeline:**
- Uses Vulkan's low-level graphics API through Vulkano
- Procedural vertex generation in vertex shader (no vertex buffers)
- Triangle-based primitive rendering for 3D shapes
- Push constants for dynamic data (3D rotation angles, geometry parameters)
- Proper swapchain recreation for window resizing
- RAII pattern for Vulkan resource management

## Application Behavior

The application renders procedurally generated 3D geometry that can be rotated in real-time. All geometry is generated mathematically in the vertex shader. Controls:
- Arrow keys: Control pitch rotation (rotation around X-axis)
- A/D keys: Control yaw rotation (rotation around Y-axis)
- Escape: Exit application  
- Runs in fullscreen borderless window mode

**Key Features:**
- No vertex data - everything generated procedurally in shaders
- Real-time 3D rotation animation
- 3D transformations with rotation matrices
- Configurable 3D geometry parameters

## Development Patterns

**Error Handling:** Currently uses `unwrap()` extensively (development-focused, not production-ready)

**Shader Integration:** GLSL shaders are embedded and compiled at build time via vulkano-shaders

**Resource Management:** Leverages Rust's ownership system for safe Vulkan resource handling

**Procedural Generation:** All vertices generated mathematically using `gl_VertexIndex`, trigonometric functions, and push constants

**Geometric Transformations:** Vertex shader handles real-time 3D rotations using rotation matrices

## Testing the Application

Run `cargo run` and test with arrow keys (pitch) and A/D keys (yaw) for 3D rotation. Visual output verification is the primary testing method as this is a graphics application. The 3D geometry should rotate smoothly in 3D space.

## Current Development Status

Currently transitioning from 2D circles to 3D geometry. Start with simple 3D shapes like triangles before building complex geometry like spheres.