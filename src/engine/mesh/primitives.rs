use glam::Vec3;

use crate::engine::Engine;

use super::{Mesh, Vertex};

pub fn make_plane_xz(engine: &Engine, num_cols: u32, num_rows: u32) -> Mesh {
    let vertex_func = |u, v| Vertex {
        in_position: Vec3::new(u - 0.5, 0.0, 0.5 - v),
        in_color: Vec3::new(u, v, 0.0),
    };

    make_plane(engine, num_cols, num_rows, vertex_func)
}

pub fn make_plane_xy(engine: &Engine, num_cols: u32, num_rows: u32) -> Mesh {
    let vertex_func = |u, v| Vertex {
        in_position: Vec3::new(u - 0.5, v - 0.5, 0.0),
        in_color: Vec3::new(u, v, 0.0),
    };

    make_plane(engine, num_cols, num_rows, vertex_func)
}

pub fn make_plane_yz(engine: &Engine, num_cols: u32, num_rows: u32) -> Mesh {
    let vertex_func = |u, v| Vertex {
        in_position: Vec3::new(0.0, v - 0.5, 0.5 - u),
        in_color: Vec3::new(u, v, 0.0),
    };

    make_plane(engine, num_cols, num_rows, vertex_func)
}

pub fn make_sharp_cube(engine: &Engine) -> Mesh {
    #[rustfmt::skip]
    let vertices = vec![
        // Front
        Vertex { in_position: Vec3::new(-0.5, -0.5,  0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5,  0.5,  0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5,  0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5, -0.5,  0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },

        // Right
        Vertex { in_position: Vec3::new( 0.5, -0.5,  0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5,  0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5, -0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5, -0.5, -0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },

        //Back
        Vertex { in_position: Vec3::new( 0.5, -0.5, -0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5, -0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5,  0.5, -0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5, -0.5, -0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },

        // Left
        Vertex { in_position: Vec3::new(-0.5, -0.5, -0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5,  0.5, -0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5,  0.5,  0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5, -0.5,  0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },

        // Top
        Vertex { in_position: Vec3::new(-0.5,  0.5,  0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5,  0.5, -0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5, -0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5,  0.5,  0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },

        // Bottom
        Vertex { in_position: Vec3::new(-0.5, -0.5, -0.5), in_color: Vec3::new(0.0, 0.0, 0.0) },
        Vertex { in_position: Vec3::new(-0.5, -0.5,  0.5), in_color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5, -0.5,  0.5), in_color: Vec3::new(1.0, 1.0, 0.0) },
        Vertex { in_position: Vec3::new( 0.5, -0.5, -0.5), in_color: Vec3::new(1.0, 0.0, 0.0) },
    ];

    #[rustfmt::skip]
    let indices = vec![
         0,  1,  3,    1,  2,  3, // Front
         4,  5,  7,    5,  6,  7, // Right
         8,  9, 11,    9, 10, 11, // Back
        12, 13, 15,   13, 14, 15, // Left
        16, 17, 19,   17, 18, 19, // Top
        20, 21, 23,   21, 22, 23, // Bottom
    ];

    Mesh::new(engine, vertices, indices)
}

fn make_plane<F>(engine: &Engine, num_cols: u32, num_rows: u32, vertex_func: F) -> Mesh
where
    F: Fn(f32, f32) -> Vertex,
{
    let num_cols = if num_cols < 2 { 2 } else { num_cols };
    let num_rows = if num_rows < 2 { 2 } else { num_rows };

    let mut vertices = Vec::new();

    for j in 0..num_rows {
        for i in 0..num_cols {
            let u = i as f32 / (num_cols - 1) as f32;
            let v = j as f32 / (num_rows - 1) as f32;

            let v = vertex_func(u, v);
            vertices.push(v);
        }
    }

    let mut indices = Vec::new();
    for j in 0..(num_rows - 1) {
        for i in 0..(num_cols - 1) {
            let i1 = i + j * num_cols;
            let i2 = i + (j + 1) * num_cols;
            let i3 = i + 1 + (j + 1) * num_cols;
            let i4 = i + 1 + j * num_cols;

            indices.extend_from_slice(&[i1, i2, i4]);
            indices.extend_from_slice(&[i2, i3, i4]);
        }
    }

    Mesh::new(engine, vertices, indices)
}
