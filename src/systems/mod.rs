use crate::components::*;
use crate::core::ClientView;
use crate::specs::prelude::*;
use crate::utils::*;

pub struct ViewSystem {
    use_cameras: bool,
    filter: Vec<String>
}


impl ViewSystem {
    pub fn new(use_cameras: bool, filter: Option<Vec<String>>) -> Self {
        ViewSystem {
            use_cameras,
            filter: filter.unwrap_or(Vec::new())
        }
    }
}

impl<'a> System<'a> for ViewSystem {
    type SystemData = (ReadConnections<'a>,
    ReadStorage<'a, Camera>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Visible>,
    WriteViewMap<'a>);

    fn run(&mut self, (connections, cameras, positions, visible, mut views): Self::SystemData) {
        if self.use_cameras {
            unimplemented!()
        } else {
            // Capture everything and load it into a single view
            let view = {
                let mut view = ClientView::new();
                for (p, v) in (&positions, &visible).join() {
                    view.sprites.push(v.sprite);
                    view.loc.push((p.x, p.y));
                }
                view
            };

            let should_filter = self.filter.len() > 0;

            for conn in &connections.connections {
                if should_filter {
                    if !self.filter.contains(&conn.key) {
                        continue;
                    }
                }
                views.insert(conn.key.clone(), view.clone());
            }
        }
    }
}