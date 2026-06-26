use serde::{Deserialize, Serialize};

pub use shared_assets::i18n::Language;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board {
    pub name: String,
    pub columns: indexmap::IndexMap<String, Column>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoardData {
    pub boards: indexmap::IndexMap<String, Board>,
    #[serde(rename = "activeBoard")]
    pub active_board: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub is_error: bool,
}

pub enum Msg {
    FetchConfigSuccess(serde_json::Value),
    FetchTasksSuccess(BoardData),
    FetchTasksError,
    VerifyPinSuccess,
    VerifyPinFailure(String),
    PinInputChanged(String),
    VerifyPin,
    Logout,
    SwitchLanguage(Language),
    ToggleTheme,
    PrintBoard,

    // Tasks management
    OpenAddTaskModal(String),
    OpenEditTaskModal(String, usize),
    TaskModalInputChanged(String),
    SaveTask,
    DeleteTask,
    DeleteTaskDirect(String, usize),
    CloseTaskModal,

    // Drag & Drop
    DragStart(String, usize, web_sys::DragEvent),
    DragOver(web_sys::DragEvent),
    Drop(String, Option<usize>, web_sys::DragEvent),

    // Toast
    DismissToast(usize),
}
