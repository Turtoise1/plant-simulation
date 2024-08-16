use super::vertex::Vertex;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct CellRenderer {
    position: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(size: Size, position: [f32; 3], lod: u16) -> Self {
        let mut cell = CellRenderer {
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
        };

        cell.update_size(size, lod);

        cell
    }

    pub fn update_size(&mut self, new_size: Size, lod: u16) {
        self.vertices = Vec::new();
        self.indices = Vec::new();

        let radius = match new_size {
            Size::FromRadius(r) => r,
            Size::FromVolume(v) => {
                // r = ((3V)/(4PI))^(1/3)
                f32::powf((3. * v) / (4. * PI), 1. / 3.)
            }
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

                self.vertices.push(Vertex {
                    position: [
                        x + self.position[0],
                        y + self.position[1],
                        z + self.position[2],
                    ],
                    color: [1., 1., 1.],
                });
            }
        }

        // Generate indices
        for i in 0..stack_count {
            for j in 0..sector_count {
                let first = (i * (sector_count + 1) + j) as u16;
                let second = (first + sector_count + 1) as u16;

                self.indices.push(first);
                self.indices.push(second);
                self.indices.push(first + 1);

                self.indices.push(second);
                self.indices.push(second + 1);
                self.indices.push(first + 1);
            }
        }
    }
}
