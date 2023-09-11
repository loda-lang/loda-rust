use super::Image;
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Action {
    Up,
    Down,
    Left,
    Right,
}

impl Action {
    #[allow(dead_code)]
    fn all() -> [Action; 4] {
        [Action::Up, Action::Down, Action::Left, Action::Right]
    }

    #[allow(dead_code)]
    fn name(&self) -> &str {
        match self {
            Action::Up => "U",
            Action::Down => "D",
            Action::Left => "L",
            Action::Right => "R",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    x: u8,
    y: u8,
}

impl State {
    #[allow(dead_code)]
    fn apply(&self, action: Action, max_x: u8, max_y: u8) -> Self {
        let mut new_x = self.x;
        let mut new_y = self.y;
        match action {
            Action::Left => new_x = new_x.saturating_sub(1),
            Action::Right => new_x = std::cmp::min((new_x as u16) + 1, max_x as u16) as u8,
            Action::Up => new_y = new_y.saturating_sub(1),
            Action::Down => new_y = std::cmp::min((new_y as u16) + 1, max_y as u16) as u8,
        }
        Self { x: new_x, y: new_y }
    }    
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct StateAction {
    state_start: State,
    action: Action,
}

#[allow(dead_code)]
fn trace_path(image: &Image, position_x: u8, position_y: u8, max_iterations: u16) -> anyhow::Result<Vec<StateAction>> {
    if image.is_empty() {
        return Err(anyhow::anyhow!("Image size must be 1x1 or bigger"));
    }
    let find_color: u8 = match image.get(position_x as i32, position_y as i32) {
        Some(color) => color,
        None => {
            return Err(anyhow::anyhow!("Position not found"));
        }
    };
    let max_x: u8 = image.width() - 1;
    let max_y: u8 = image.height() - 1;

    let mut x: u8 = position_x;
    let mut y: u8 = position_y;
    let mut visited = HashSet::<State>::new();
    let mut state_action_vec = Vec::<StateAction>::new();

    for _iteration in 0..max_iterations {
        let state = State { x, y };
        visited.insert(state);

        let mut count: usize = 0;
        for action in Action::all() {
            let new_s: State = state.apply(action, max_x, max_y);
            if new_s == state {
                // println!("state: {:?} action: {:?} same state", new_s, action);
                continue;
            }
            if visited.contains(&new_s) {
                // println!("state: {:?} action: {:?} already visited", new_s, action);
                continue;
            }
            let color: u8 = image.get(new_s.x as i32, new_s.y as i32).unwrap_or(255);
            if color != find_color {
                // println!("state: {:?} action: {:?} not following the path. color: {} != find_value: {}", new_s, action, color, find_value);
                continue;
            }
            // follow the path indicated by the color
            // println!("state: {:?} action: {:?} stay on the path. color: {}", new_s, action, color);
            x = new_s.x;
            y = new_s.y;

            state_action_vec.push(StateAction { state_start: state, action });
            count += 1;
        }
        if count > 1 {
            return Err(anyhow::anyhow!("Ambiguous action. Multiple ways to go"));
        }
        if count == 0 {
            // reached the end of the path
            break;
        }
    }
    Ok(state_action_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::arc_json_model;
    use crate::arc::arc_work_model::{Pair, Task};
    use crate::arc::ImageFind;

    fn verbose_trace_path(pair: &Pair, find_color: u8) -> anyhow::Result<String> {
        let input: &Image = &pair.input.image;
    
        let needle = Image::color(1, 1, find_color);
        let (input_position_x, input_position_y) = match input.find_first(&needle)? {
            Some((x, y)) => (x, y),
            None => {
                return Err(anyhow::anyhow!("Needle not found"));
            }
        };
    
        let state_action_vec: Vec<StateAction> = trace_path(&pair.output.image, input_position_x, input_position_y, 100)?;
        let action_name_vec: Vec<&str> = state_action_vec.iter().map(|state_action| state_action.action.name()).collect();
        let action_names: String = action_name_vec.join("");
    
        let s = format!("x: {} y: {} actions: {}", input_position_x, input_position_y, action_names);
        Ok(s)
    }
    
    #[test]
    fn test_10000_trace_path() -> anyhow::Result<()> {
        let json_task: arc_json_model::Task = arc_json_model::Task::load_testdata("96a8c0cd")?;
        let task: Task = Task::try_from(&json_task)?;    
        let pair: &Pair = &task.pairs[0];
        let actual: String = verbose_trace_path(pair, 2)?;
        let expected = "x: 0 y: 3 actions: RRRRRDDRRRUURRRRDDRRRRUURRRUURR";
        assert_eq!(actual, expected);
        Ok(())
    }
    
    #[test]
    fn test_10001_trace_path() -> anyhow::Result<()> {
        let json_task: arc_json_model::Task = arc_json_model::Task::load_testdata("96a8c0cd")?;
        let task: Task = Task::try_from(&json_task)?;    
        let pair: &Pair = &task.pairs[2];
        let actual: String = verbose_trace_path(pair, 2)?;
        let expected = "x: 5 y: 0 actions: DRRDDRRDDLDDDLLDDRRRDDDLLLDDDLLLDDLDDD";
        assert_eq!(actual, expected);
        Ok(())
    }
}
