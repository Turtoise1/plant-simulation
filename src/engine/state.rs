use std::sync::Arc;

use cgmath::{EuclideanSpace, Point3, Vector3};
use winit::{dpi::PhysicalPosition, window::Window};

use crate::shared::{
    cell::{Cell, CellEvent, CellEventType, EventSystem},
    math::{distance, line_plane_intersection, Line, Line2PlaneClassification, Plane},
};

pub struct ApplicationState {
    window: Arc<Window>,
    cells: Arc<Vec<Cell>>,
    cell_events: Arc<EventSystem>,
    pub mouse_position: Option<PhysicalPosition<f64>>,
}

impl ApplicationState {
    pub async fn new(
        window: Arc<Window>,
        cells: Arc<Vec<Cell>>,
        cell_events: Arc<EventSystem>,
    ) -> Self {
        ApplicationState {
            window,
            cells,
            cell_events,
            mouse_position: None,
        }
    }

    // pub fn screen_pos_2_select_ray(&self, screen_pos: &PhysicalPosition<f64>) -> Line<f32> {
    //     let view_projection_matrix = self.camera.build_view_projection_matrix();
    //     let inverted = view_projection_matrix.invert().unwrap();
    //     let width = self.window.inner_size().width as f64;
    //     let height = self.window.inner_size().height as f64;
    //     let x = screen_pos.x;
    //     let y = screen_pos.y;
    //     // Convert screen position to NDC
    //     let ndc_x = ((2.0 * x) / width - 1.0) as f32;
    //     let ndc_y = (1.0 - (2.0 * y) / height) as f32;

    //     // Clip space coordinates
    //     let near_clip = Vector4::new(ndc_x, ndc_y, -1.0, 1.0);
    //     let far_clip = Vector4::new(ndc_x, ndc_y, 1.0, 1.0);

    //     // Transform to world space
    //     let near_world = inverted * near_clip;
    //     let far_world = inverted * far_clip;

    //     // Perspective divide to get 3D coordinates
    //     let near_point = Point3::new(
    //         near_world.x / near_world.w,
    //         near_world.y / near_world.w,
    //         near_world.z / near_world.w,
    //     );

    //     let far_point = Point3::new(
    //         far_world.x / far_world.w,
    //         far_world.y / far_world.w,
    //         far_world.z / far_world.w,
    //     );

    //     // Compute ray direction
    //     let ray_dir = (far_point - near_point).normalize();
    //     Line {
    //         pos: near_point.to_vec(),
    //         dir: ray_dir,
    //     }
    // }

    pub fn select_cells(&self, select_ray: Line<f32>) {
        for cell in self.cells.iter() {
            let renderer = cell.renderer.read().unwrap();
            let cell_pos = renderer.position();
            let cell_pos = Vector3::new(cell_pos.x, cell_pos.y, cell_pos.z);
            let cell_plane = Plane::<f32> {
                pos: cell_pos,
                normal: select_ray.dir,
            };
            match line_plane_intersection(&select_ray, &cell_plane) {
                Line2PlaneClassification::Parallel => {
                    panic!("This should not happen because the plane should be orthogonal to the select ray!")
                }
                Line2PlaneClassification::Intersects(intersection_point) => {
                    if distance(
                        &(Point3::origin() + cell_pos),
                        &(Point3::origin() + intersection_point),
                    ) < *renderer.radius()
                    {
                        println!("Intersection with cell {}", renderer.cell_id());
                        self.cell_events.notify(Arc::new(CellEvent {
                            id: renderer.cell_id(),
                            event_type: CellEventType::Mark(Option::None),
                        }));
                    }
                }
            }
        }
    }
}
