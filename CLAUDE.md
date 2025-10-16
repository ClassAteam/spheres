# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based graphics application workspace containing two separate implementations using the Vulkan API through the Vulkano library. Both applications render animated geometric shapes in fullscreen mode, demonstrating different approaches to real-time graphics programming.

**Two Implementations:**
- **spheres-lines**: Original rotating lines version with static vertex data
- **spheres-circles**: Procedural circle generation in vertex shader

## Build and Development Commands

**Workspace Commands:**
```bash
cargo check                    # Check all binaries
cargo build                    # Build all binaries
cargo build --release         # Release build all
```

**Individual Binary Commands:**
```bash
cargo run -p spheres-lines     # Run rotating lines version (original)
cargo run -p spheres-circles   # Run procedural circles version
cargo check -p spheres-lines   # Check lines binary only
cargo check -p spheres-circles # Check circles binary only
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
- `model.rs` - Vertex data and geometry definitions
- `vert.glsl`/`frag.glsl` - Vertex and fragment shaders with push constants

**Rendering Pipeline:**
- Uses Vulkan's low-level graphics API through Vulkano
- Line-based primitive rendering (PrimitiveTopology::LineList)
- Push constants for dynamic data (rotation angles, aspect ratio)
- Proper swapchain recreation for window resizing
- RAII pattern for Vulkan resource management

## Application Behavior

The application renders rotating geometric patterns consisting of a rectangular frame and two rotating lines. Controls:
- Arrow keys: Control rotation of geometric elements
- Escape: Exit application
- Runs in fullscreen borderless window mode

## Development Patterns

**Error Handling:** Currently uses `unwrap()` extensively (development-focused, not production-ready)

**Shader Integration:** GLSL shaders are embedded and compiled at build time via vulkano-shaders

**Resource Management:** Leverages Rust's ownership system for safe Vulkan resource handling

**Geometric Transformations:** Vertex shader handles real-time transformations with aspect ratio correction

## Testing the Application

Run `cargo run` and test with arrow keys for rotation. Visual output verification is the primary testing method as this is a graphics application.