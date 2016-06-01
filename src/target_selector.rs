
use glorious::{Behavior, Renderer, Sprite};
use common::{State, Message, MARKER_PATH};
use unit::Unit;
use grid::Tile;

#[derive(Debug)]
pub struct TargetSelector {
    unit: Unit,
    cell: (u32, u32),
    origin: (u32, u32),
    selected: usize,
    cell_size: (u32, u32),
    grid_size: (u32, u32),
    targets: Vec<((u32, u32), Tile)>,
}

impl TargetSelector {
    pub fn new(unit: Unit,
               cell: (u32, u32),
               origin: (u32, u32),
               grid_size: (u32, u32),
               cell_size: (u32, u32),
               targets: Vec<((u32, u32), Tile)>)
               -> TargetSelector {
        assert!(!targets.is_empty(), "No targets given to selector");
        TargetSelector {
            unit: unit,
            cell: cell,
            origin: origin,
            selected: 0,
            grid_size: grid_size,
            cell_size: cell_size,
            targets: targets,
        }
    }
}

impl<'a> Behavior<State<'a>> for TargetSelector {
    type Message = Message;

    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            Confirm => {
                let selected = self.targets[self.selected].0;
                info!("Attacking target at {:?}", selected);
                state.break_modal(queue);
                queue.push(UnitSpent(self.cell));
                queue.push(Deselect);
                queue.push(ShowCursor);
            }
            Cancel => {
                state.break_modal(queue);
                queue.push(MoveUnit(self.cell, self.origin));
                queue.push(MoveUnitAndAct(self.origin, self.cell));
            }
            MoveCursorDown | MoveCursorRight => {
                self.selected = (self.selected + 1) % self.targets.len();
            }
            MoveCursorUp | MoveCursorLeft => {
                self.selected = (self.selected + self.targets.len() - 1) % self.targets.len();
            }
            _ => {}
        }
    }

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        let (col, row) = self.targets[self.selected].0;
        let x = (col * self.cell_size.0) as i32;
        let grid_height = self.grid_size.1 * self.cell_size.1;
        let y = (grid_height - self.cell_size.1 - (row * self.cell_size.1)) as i32;
        let sprite = Sprite::new(state.resources.texture(MARKER_PATH), None);
        sprite.render(renderer, x, y, Some(self.cell_size));
    }
}
