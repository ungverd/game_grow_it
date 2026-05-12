struct TreeUnit {
    left: u32,
    right: u32,
    repeats: u32,
}

struct Monkey {
    x_pos: f64,
    is_running: bool,
}

struct GameState {
    tree: Vec<TreeUnit>,
    monkey: Monkey,
}

impl GameState {
    fn create_new_tree_unit(&mut self, left:u32, right:u32) {
        self.tree.push(TreeUnit{left, right, repeats: 1})
    }

    pub fn grow_tree(&mut self, left: u32, right: u32) {
        let last = self.tree.last_mut();
        match last {
            Some(last) => {
                if last.left == left && last.right == right {
                    last.repeats += 1;
                } else {
                    self.create_new_tree_unit(left, right);
                }
            }
            None => { self.create_new_tree_unit(left, right); }
        }
    }

    pub fn undo_grow_tree(&mut self) {
        let last = self.tree.last_mut();
        match last {
            Some(last) => {
                if last.repeats > 1 {
                    last.repeats -= 1;
                } else {
                    self.tree.pop();
                }
            }
            None => {}
        }
    }
}