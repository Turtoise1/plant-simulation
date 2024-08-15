use super::vertex::Vertex;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct CellRenderer {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl CellRenderer {
    pub fn new(radius: f32, position: [f32; 3], lod: u16) -> Self {
        let mut cell = CellRenderer {
            vertices: Vec::new(),
            indices: Vec::new(),
        };

        let sector_count = lod * 2;
        let stack_count = lod;

        let sector_step = 2.0 * PI / sector_count as f32;
        let stack_step = PI / stack_count as f32;

        for i in 0..=stack_count {
            let stack_angle = PI / 2.0 - i as f32 * stack_step;
            let xy = radius * stack_angle.cos();
            let z = radius * stack_angle.sin();

            for j in 0..=sector_count {
                let sector_angle = j as f32 * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                cell.vertices.push(Vertex {
                    position: [x + position[0], y + position[1], z + position[2]],
                    color: [1., 1., 1.],
                });
            }
        }

        // Generate indices
        for i in 0..stack_count {
            for j in 0..sector_count {
                let first = (i * (sector_count + 1) + j) as u16;
                let second = (first + sector_count + 1) as u16;

                cell.indices.push(first);
                cell.indices.push(second);
                cell.indices.push(first + 1);

                cell.indices.push(second);
                cell.indices.push(second + 1);
                cell.indices.push(first + 1);
            }
        }

        cell
    }
}
