# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace containing Vulkan-based graphics applications using the `vulkano` library for rendering 3D content with `winit` for window management.

### Workspace Members

- **spheres-circles**: Main 3D rendering application with interactive rotation controls and procedural geometry generation
- **cube**: Experimental Vulkan rendering setup (early development stage)

## Build and Run Commands

```bash
# Build all workspace members
cargo build

# Build specific member
cargo build -p spheres-circles
cargo build -p cube

# Run specific application
cargo run -p spheres-circles
cargo run -p cube

# Build in release mode
cargo build --release -p spheres-circles
```

## Architecture

### spheres-circles Application

The main application follows a **two-tier context separation** pattern:

1. **Device-Independent Context** (`RenderContext` in `render_context.rs`):
   - Owns Vulkan instance, physical/logical device, and queue
   - Created once during event loop initialization
   - Lives independently of window lifecycle
   - Contains command buffer allocator

2. **Window-Dependent Context** (`WindowDependentContext` in `window_context.rs`):
   - Created in `resumed()` lifecycle method when window surface becomes available
   - Owns swapchain, render pass, framebuffers, graphics pipeline
   - Handles swapchain recreation on resize
   - Contains the rendering control state (`Control`)

This separation is critical because:
- On some platforms (iOS, Android, Wayland), window surfaces may be created/destroyed during app lifecycle
- Vulkan instance and device can persist across surface recreation
- The `ApplicationHandler::resumed()` pattern in winit 0.30 requires deferred surface creation

### Key Components

- **model.rs**: Defines `SphereParams` for procedural sphere generation parameters
- **control.rs**: Manages user input state (rotation angles on X/Y/Z axes)
- **window_context.rs**:
  - Manages entire Vulkan rendering pipeline per window
  - Compiles GLSL shaders at build time via `vulkano_shaders` macro
  - Uses push constants to pass rotation and sphere parameters to shaders
- **Shaders**:
  - `vert.glsl`: Vertex shader with procedural geometry generation and 3D rotation matrices
  - `frag.glsl`: Fragment shader for color output

### User Controls

The application uses keyboard input for 3D rotation:
- `W`: Rotate up (rotation_x += 0.02)
- `ArrowLeft`: Rotate down (rotation_x -= 0.02)
- `A`: Rotate left (rotation_y -= 0.02)
- `D`: Rotate right (rotation_y += 0.02)
- `Escape`: Exit application

### Rendering Pipeline

1. Event loop triggers `RedrawRequested`
2. `RenderContext::draw()` delegates to `WindowDependentContext::redraw()`
3. Swapchain image acquisition
4. Command buffer creation with push constants (rotation + sphere params)
5. Draw call (currently renders 3 vertices as a triangle)
6. Command buffer execution and present

### Important Notes

- The application currently renders a simple rotating triangle despite having sphere parameter infrastructure
- Shaders receive sphere generation parameters but don't yet use them for actual sphere rendering
- Edition is set to "2024" which requires Rust 1.85+ (currently in beta/nightly as of Dec 2024)
- All window creation uses borderless fullscreen mode
