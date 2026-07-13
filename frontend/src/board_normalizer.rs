use crate::types::{Board, BoardData, Column};

pub fn normalize_board_data(mut data: BoardData) -> (BoardData, bool) {
    let mut needs_save = false;
    if data.boards.is_empty() {
        let mut cols = indexmap::IndexMap::new();
        cols.insert(
            "todo".to_string(),
            Column {
                name: "To Do".to_string(),
                tasks: vec![],
            },
        );
        cols.insert(
            "doing".to_string(),
            Column {
                name: "Doing".to_string(),
                tasks: vec![],
            },
        );
        cols.insert(
            "done".to_string(),
            Column {
                name: "Done".to_string(),
                tasks: vec![],
            },
        );
        let new_board = Board {
            name: "Work".to_string(),
            columns: cols,
        };
        data.boards.insert("work".to_string(), new_board);
        data.active_board = "work".to_string();
        needs_save = true;
    }

    for board in data.boards.values_mut() {
        let mut new_cols = indexmap::IndexMap::new();
        let mut todo_tasks = vec![];
        let mut doing_tasks = vec![];
        let mut done_tasks = vec![];

        let old_cols = std::mem::take(&mut board.columns);
        for (id, col) in old_cols {
            match id.as_str() {
                "todo" => todo_tasks.extend(col.tasks),
                "doing" => doing_tasks.extend(col.tasks),
                "done" => done_tasks.extend(col.tasks),
                _ => {
                    todo_tasks.extend(col.tasks);
                    needs_save = true;
                }
            }
        }

        new_cols.insert(
            "todo".to_string(),
            Column {
                name: "To Do".to_string(),
                tasks: todo_tasks,
            },
        );
        new_cols.insert(
            "doing".to_string(),
            Column {
                name: "Doing".to_string(),
                tasks: doing_tasks,
            },
        );
        new_cols.insert(
            "done".to_string(),
            Column {
                name: "Done".to_string(),
                tasks: done_tasks,
            },
        );

        board.columns = new_cols;
    }
    (data, needs_save)
}
