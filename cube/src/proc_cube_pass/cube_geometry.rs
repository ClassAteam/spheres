use crate::proc_cube_pass::proc_cube_pass::Position;

pub struct CubeGeometry {
    faces: Vec<Face>,
}

impl CubeGeometry {
    pub fn new() -> Self {
        let mut faces = Vec::new();

        faces.push(Face::new(
            Orientation::Back,
            Rectangle::new(
                Vertex::new(-1.0, 1.0, -1.0),
                Vertex::new(1.0, 1.0, -1.0),
                Vertex::new(1.0, -1.0, -1.0),
                Vertex::new(-1.0, -1.0, -1.0),
            ),
            Color::new(1.0, 0.0, 0.0),
        ));
        faces.push(Face::new(
            Orientation::Front,
            Rectangle::new(
                Vertex::new(-1.0, 1.0, 1.0),
                Vertex::new(1.0, 1.0, 1.0),
                Vertex::new(1.0, -1.0, 1.0),
                Vertex::new(-1.0, -1.0, 1.0),
            ),
            Color::new(0.0, 1.0, 0.0),
        ));
        faces.push(Face::new(
            Orientation::Left,
            Rectangle::new(
                Vertex::new(-1.0, 1.0, -1.0),
                Vertex::new(-1.0, -1.0, -1.0),
                Vertex::new(-1.0, -1.0, 1.0),
                Vertex::new(-1.0, 1.0, 1.0),
            ),
            Color::new(0.0, 0.0, 1.0),
        ));
        faces.push(Face::new(
            Orientation::Right,
            Rectangle::new(
                Vertex::new(1.0, 1.0, -1.0),
                Vertex::new(1.0, 1.0, 1.0),
                Vertex::new(1.0, -1.0, 1.0),
                Vertex::new(1.0, -1.0, -1.0),
            ),
            Color::new(1.0, 1.0, 0.0),
        ));
        faces.push(Face::new(
            Orientation::Bottom,
            Rectangle::new(
                Vertex::new(-1.0, 1.0, -1.0),
                Vertex::new(-1.0, 1.0, 1.0),
                Vertex::new(1.0, 1.0, 1.0),
                Vertex::new(1.0, 1.0, -1.0),
            ),
            Color::new(1.0, 0.0, 1.0),
        ));
        faces.push(Face::new(
            Orientation::Top,
            Rectangle::new(
                Vertex::new(-1.0, -1.0, -1.0),
                Vertex::new(1.0, -1.0, -1.0),
                Vertex::new(1.0, -1.0, 1.0),
                Vertex::new(-1.0, -1.0, 1.0),
            ),
            Color::new(0.0, 1.0, 1.0),
        ));

        Self { faces }
    }
    pub fn generate_cube_vertices(&self) -> Vec<Position> {
        let mut vertices = Vec::with_capacity(24); // 6 faces *  4vertices
        for face in &self.faces {
            let color = face.color();
            let positions = face.vertices();

            for position in &positions {
                vertices.push(Position {
                    position: *position,
                    color,
                });
            }
        }
        vertices
    }

    pub fn generate_indices(&self) -> Vec<u16> {
        let mut indices = Vec::with_capacity(36); // 6 faces × 6 indices
        for (face_index, face) in self.faces.iter().enumerate() {
            let base = (face_index * 4) as u16; // Each face has 4 vertices
            indices.extend(face.indices().iter().map(|&idx| base + idx));
        }
        indices
    }
}

impl Face {
    pub fn new(face: Orientation, rect: Rectangle, color: Color) -> Self {
        Self {
            place: face,
            rect: rect,
            color: color,
        }
    }

    pub fn color(&self) -> [f32; 3] {
        [self.color.r, self.color.g, self.color.b]
    }

    pub fn vertices(&self) -> [[f32; 3]; 4] {
        [
            self.rect.bl.coords,
            self.rect.br.coords,
            self.rect.tr.coords,
            self.rect.tl.coords,
        ]
    }

    pub fn indices(&self) -> Vec<u16> {
        vec![3, 2, 0, 0, 2, 1]
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    coords: [f32; 3],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { coords: [x, y, z] }
    }
}

struct Rectangle {
    bl: Vertex,
    br: Vertex,
    tr: Vertex,
    tl: Vertex,
}

impl Rectangle {
    pub fn new(bl: Vertex, br: Vertex, tr: Vertex, tl: Vertex) -> Self {
        Self { bl, br, tr, tl }
    }
}

struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

struct Face {
    place: Orientation,
    rect: Rectangle,
    color: Color,
}
#[derive(Debug, Clone, Copy)]
enum Orientation {
    Back,   // -Z
    Front,  // +Z
    Left,   // -X
    Right,  // +X
    Bottom, // +Y
    Top,    // -Y
}
