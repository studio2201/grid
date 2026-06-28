use crate::app::App;
use crate::i18n::get_translations;
use crate::types::*;
use yew::prelude::*;

impl App {
    pub fn render_board(&self, board: &Board, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        html! {
            <main>
                <div class="board">
                    {for board.columns.iter().map(|(col_id, col)| {
                        let col_id = col_id.clone();
                        let col_id_drop = col_id.clone();
                        let col_id_add = col_id.clone();
                        let col_id_tasks = col_id.clone();
                        html! {
                            <div class="column">
                                <div class="column-header-box">
                                    <h2 class="column-name">
                                        {&col.name}
                                    </h2>
                                </div>
                                <div
                                    class="column-tasks-box"
                                    ondragover={ctx.link().callback(Msg::DragOver)}
                                    ondrop={ctx.link().callback(move |e| Msg::Drop(col_id_drop.clone(), None, e))}
                                >
                                    <div class="tasks">
                                        {for col.tasks.iter().enumerate().map({
                                            let col_id_tasks = col_id_tasks.clone();
                                            move |(idx, task)| {
                                                let col_id_drag = col_id_tasks.clone();
                                                let col_id_drop = col_id_tasks.clone();
                                                let col_id_edit = col_id_tasks.clone();
                                                let col_id_delete = col_id_tasks.clone();

                                                let on_delete_click = ctx.link().callback(move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    Msg::DeleteTaskDirect(col_id_delete.clone(), idx)
                                                });

                                                html! {
                                                    <div
                                                        class="task"
                                                        draggable="true"
                                                        ondragstart={ctx.link().callback(move |e| Msg::DragStart(col_id_drag.clone(), idx, e))}
                                                        ondragover={ctx.link().callback(Msg::DragOver)}
                                                        ondrop={ctx.link().callback(move |e| Msg::Drop(col_id_drop.clone(), Some(idx), e))}
                                                        ondblclick={ctx.link().callback(move |_| Msg::OpenEditTaskModal(col_id_edit.clone(), idx))}
                                                    >
                                                        <div class="move-indicator">{"⋮⋮"}</div>
                                                        <div class="task-content">
                                                            <span class="task-text">{task}</span>
                                                        </div>
                                                        <button
                                                            class="task-delete-btn"
                                                            onclick={on_delete_click}
                                                            title={tr.delete.to_string()}
                                                        >
                                                            {"×"}
                                                        </button>
                                                    </div>
                                                }
                                            }
                                        })}
                                    </div>
                                </div>
                                if col_id == "todo" {
                                    <button
                                        class="add-task"
                                        onclick={ctx.link().callback(move |_| Msg::OpenAddTaskModal(col_id_add.clone()))}
                                    >
                                        {tr.add_task}
                                    </button>
                                }
                            </div>
                        }
                    })}
                </div>
            </main>
        }
    }
}
