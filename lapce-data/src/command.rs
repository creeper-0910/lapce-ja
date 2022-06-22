use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use druid::{FileInfo, Point, Rect, Selector, SingleUse, Size, WidgetId, WindowId};
use indexmap::IndexMap;
use lapce_core::buffer::DiffLines;
use lapce_core::command::{
    EditCommand, FocusCommand, MotionModeCommand, MoveCommand, MultiSelectionCommand,
};
use lapce_core::syntax::Syntax;
use lapce_rpc::{
    buffer::BufferId, file::FileNodeItem, plugin::PluginDescription,
    source_control::DiffInfo, style::Style, terminal::TermId,
};
use lsp_types::{
    CodeActionOrCommand, CodeActionResponse, CompletionItem, CompletionResponse,
    Location, Position, ProgressParams, PublishDiagnosticsParams, TextEdit,
};
use serde_json::Value;
use strum::{self, EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumMessage, EnumString, IntoStaticStr};
use xi_rope::{spans::Spans, Rope};

use crate::alert::AlertContentData;
use crate::data::LapceWorkspace;
use crate::document::BufferContent;
use crate::menu::MenuKind;
use crate::rich_text::RichText;
use crate::{
    data::{EditorTabChild, SplitContent},
    editor::EditorLocation,
    keypress::{KeyMap, KeyPress},
    palette::{PaletteItem, PaletteType},
    proxy::ProxyStatus,
    search::Match,
    split::{SplitDirection, SplitMoveDirection},
};

pub const LAPCE_OPEN_FOLDER: Selector<FileInfo> = Selector::new("lapce.open-folder");
pub const LAPCE_OPEN_FILE: Selector<FileInfo> = Selector::new("lapce.open-file");
pub const LAPCE_SAVE_FILE_AS: Selector<FileInfo> =
    Selector::new("lapce.save-file-as");
pub const LAPCE_COMMAND: Selector<LapceCommand> = Selector::new("lapce.new-command");
pub const LAPCE_UI_COMMAND: Selector<LapceUICommand> =
    Selector::new("lapce.ui_command");

#[derive(Clone, Debug)]
pub struct LapceCommand {
    pub kind: CommandKind,
    pub data: Option<Value>,
}

#[derive(Clone, Debug)]
pub enum CommandKind {
    Workbench(LapceWorkbenchCommand),
    Edit(EditCommand),
    Move(MoveCommand),
    Focus(FocusCommand),
    MotionMode(MotionModeCommand),
    MultiSelection(MultiSelectionCommand),
}

impl CommandKind {
    pub fn desc(&self) -> Option<&'static str> {
        match &self {
            CommandKind::Workbench(cmd) => cmd.get_message(),
            CommandKind::Edit(cmd) => cmd.get_message(),
            CommandKind::Move(cmd) => cmd.get_message(),
            CommandKind::Focus(cmd) => cmd.get_message(),
            CommandKind::MotionMode(cmd) => cmd.get_message(),
            CommandKind::MultiSelection(cmd) => cmd.get_message(),
        }
    }

    pub fn str(&self) -> &'static str {
        match &self {
            CommandKind::Workbench(cmd) => cmd.into(),
            CommandKind::Edit(cmd) => cmd.into(),
            CommandKind::Move(cmd) => cmd.into(),
            CommandKind::Focus(cmd) => cmd.into(),
            CommandKind::MotionMode(cmd) => cmd.into(),
            CommandKind::MultiSelection(cmd) => cmd.into(),
        }
    }
}

impl LapceCommand {
    pub const PALETTE: &'static str = "palette";

    pub fn is_palette_command(&self) -> bool {
        if let CommandKind::Workbench(cmd) = &self.kind {
            match cmd {
                LapceWorkbenchCommand::Palette
                | LapceWorkbenchCommand::PaletteLine
                | LapceWorkbenchCommand::PaletteSymbol
                | LapceWorkbenchCommand::PaletteCommand
                | LapceWorkbenchCommand::ChangeTheme
                | LapceWorkbenchCommand::ConnectSshHost
                | LapceWorkbenchCommand::ConnectWsl
                | LapceWorkbenchCommand::PaletteWorkspace => return true,
                _ => {}
            }
        }

        false
    }
}

#[derive(PartialEq)]
pub enum CommandExecuted {
    Yes,
    No,
}

pub fn lapce_internal_commands() -> IndexMap<String, LapceCommand> {
    let mut commands = IndexMap::new();

    for c in LapceWorkbenchCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::Workbench(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    for c in EditCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::Edit(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    for c in MoveCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::Move(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    for c in FocusCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::Focus(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    for c in MotionModeCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::MotionMode(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    for c in MultiSelectionCommand::iter() {
        let command = LapceCommand {
            kind: CommandKind::MultiSelection(c.clone()),
            data: None,
        };
        commands.insert(c.to_string(), command);
    }

    commands
}

#[derive(
    Display,
    EnumString,
    EnumIter,
    Clone,
    PartialEq,
    Debug,
    EnumMessage,
    IntoStaticStr,
)]
pub enum LapceWorkbenchCommand {
    #[strum(serialize = "enable_modal_editing")]
    #[strum(message = "modal編集の有効化")]
    EnableModal,

    #[strum(serialize = "disable_modal_editing")]
    #[strum(message = "modal編集の無効化")]
    DisableModal,

    #[strum(serialize = "open_folder")]
    #[strum(message = "folderを開く")]
    OpenFolder,

    #[strum(serialize = "close_folder")]
    #[strum(message = "folderを閉じる")]
    CloseFolder,

    #[strum(serialize = "open_file")]
    #[strum(message = "fileを開く")]
    OpenFile,

    #[strum(serialize = "change_theme")]
    #[strum(message = "themeの変更")]
    ChangeTheme,

    #[strum(serialize = "open_settings")]
    #[strum(message = "設定を開く")]
    OpenSettings,

    #[strum(serialize = "open_settings_file")]
    #[strum(message = "設定fileを開く")]
    OpenSettingsFile,

    #[strum(serialize = "open_keyboard_shortcuts")]
    #[strum(message = "keyboad shortcutを開く")]
    OpenKeyboardShortcuts,

    #[strum(serialize = "open_keyboard_shortcuts_file")]
    #[strum(message = "keyboad shortcutのfileを開く")]
    OpenKeyboardShortcutsFile,

    #[strum(serialize = "open_log_file")]
    #[strum(message = "log fileを開く")]
    OpenLogFile,

    #[strum(serialize = "close_window_tab")]
    #[strum(message = "現在のWindow Tabを閉じる")]
    CloseWindowTab,

    #[strum(serialize = "new_window_tab")]
    #[strum(message = "新しいWindow Tabを作成する")]
    NewWindowTab,

    #[strum(serialize = "next_window_tab")]
    #[strum(message = "次のWindow Tabへ")]
    NextWindowTab,

    #[strum(serialize = "previous_window_tab")]
    #[strum(message = "前のWindow Tabへ")]
    PreviousWindowTab,

    #[strum(serialize = "reload_window")]
    #[strum(message = "windowの再読み込み")]
    ReloadWindow,

    #[strum(message = "新規window")]
    #[strum(serialize = "new_window")]
    NewWindow,

    #[strum(message = "新規file")]
    #[strum(serialize = "new_file")]
    NewFile,

    #[strum(serialize = "connect_ssh_host")]
    #[strum(message = "SSH hostに接続")]
    ConnectSshHost,

    #[strum(serialize = "connect_wsl")]
    #[strum(message = "WSLに接続")]
    ConnectWsl,

    #[strum(serialize = "disconnect_remote")]
    #[strum(message = "remote接続を切断")]
    DisconnectRemote,

    #[strum(serialize = "palette.line")]
    PaletteLine,

    #[strum(serialize = "palette")]
    #[strum(message = "fileへ移動")]
    Palette,

    #[strum(serialize = "palette.symbol")]
    PaletteSymbol,

    #[strum(message = "command palette")]
    #[strum(serialize = "palette.command")]
    PaletteCommand,

    #[strum(message = "最近のworkspaceを開く")]
    #[strum(serialize = "palette.workspace")]
    PaletteWorkspace,

    #[strum(serialize = "source_control.checkout_branch")]
    CheckoutBranch,

    #[strum(serialize = "toggle_maximized_panel")]
    ToggleMaximizedPanel,

    #[strum(serialize = "hide_panel")]
    HidePanel,

    #[strum(serialize = "show_panel")]
    ShowPanel,

    /// Toggles the panel passed in parameter.
    #[strum(serialize = "toggle_panel_focus")]
    TogglePanelFocus,

    /// Toggles the panel passed in parameter.
    #[strum(serialize = "toggle_panel_visual")]
    TogglePanelVisual,

    // Focus toggle commands
    #[strum(message = "Toggle Terminal Focus")]
    #[strum(serialize = "toggle_terminal_focus")]
    ToggleTerminalFocus,

    #[strum(serialize = "toggle_source_control_focus")]
    ToggleSourceControlFocus,

    #[strum(message = "pluginのfocus切り替え")]
    #[strum(serialize = "toggle_plugin_focus")]
    TogglePluginFocus,

    #[strum(message = "file explorerのfocus切り替え")]
    #[strum(serialize = "toggle_file_explorer_focus")]
    ToggleFileExplorerFocus,

    #[strum(message = "ploblemのfocus切り替え")]
    #[strum(serialize = "toggle_problem_focus")]
    ToggleProblemFocus,

    #[strum(message = "searchのfocus切り替え")]
    #[strum(serialize = "toggle_search_focus")]
    ToggleSearchFocus,

    // Visual toggle commands
    #[strum(serialize = "toggle_terminal_visual")]
    ToggleTerminalVisual,

    #[strum(serialize = "toggle_source_control_visual")]
    ToggleSourceControlVisual,

    #[strum(serialize = "toggle_plugin_visual")]
    TogglePluginVisual,

    #[strum(serialize = "toggle_file_explorer_visual")]
    ToggleFileExplorerVisual,

    #[strum(serialize = "toggle_problem_visual")]
    ToggleProblemVisual,

    #[strum(serialize = "toggle_search_visual")]
    ToggleSearchVisual,

    #[strum(serialize = "focus_editor")]
    FocusEditor,

    #[strum(serialize = "focus_terminal")]
    FocusTerminal,

    #[strum(serialize = "source_control_commit")]
    SourceControlCommit,

    #[strum(serialize = "export_current_theme_settings")]
    #[strum(message = "current settingsをtheme fileにexport")]
    ExportCurrentThemeSettings,

    #[strum(serialize = "install_theme")]
    #[strum(message = "current theme fileをinstall")]
    InstallTheme,
}

#[derive(Debug)]
pub enum EnsureVisiblePosition {
    // Move the view so the cursor line will be at the center of the window.  If
    // the cursor is near the beginning of the buffer, the view might not
    // change.
    CenterOfWindow,
    // Cursor will be at the top edge, down by a margin.
    TopOfWindow,
    // Cursor will be at the bottom edge, up by a margin.  If the cursor is near
    // the beginning of the buffer, the view might not change.
    BottomOfWindow,
}

pub enum LapceUICommand {
    InitChildren,
    InitTerminalPanel(bool),
    ReloadConfig,
    InitBufferContent {
        path: PathBuf,
        content: Rope,
        locations: Vec<(WidgetId, EditorLocation)>,
    },
    OpenFileChanged {
        path: PathBuf,
        content: Rope,
    },
    ReloadBuffer {
        path: PathBuf,
        rev: u64,
        content: Rope,
    },
    LoadBufferHead {
        path: PathBuf,
        version: String,
        content: Rope,
    },
    LoadBufferAndGoToPosition {
        path: PathBuf,
        content: String,
        editor_view_id: WidgetId,
        location: EditorLocation,
    },
    ShowAlert(AlertContentData),
    ShowMenu(Point, Arc<Vec<MenuKind>>),
    UpdateSearchInput(String),
    UpdateSearch(String),
    GlobalSearchResult(String, Arc<HashMap<PathBuf, Vec<Match>>>),
    CancelFilePicker,
    SetWorkspace(LapceWorkspace),
    SetTheme(String, bool),
    UpdateKeymap(KeyMap, Vec<KeyPress>),
    OpenFile(PathBuf),
    OpenFileDiff(PathBuf, String),
    CancelCompletion(usize),
    ResolveCompletion(BufferId, u64, usize, Box<CompletionItem>),
    UpdateCompletion(usize, String, CompletionResponse),
    UpdateHover(usize, Arc<Vec<RichText>>),
    UpdateCodeActions(PathBuf, u64, usize, CodeActionResponse),
    CancelPalette,
    RunCodeAction(CodeActionOrCommand),
    ShowCodeActions(Option<Point>),
    Hide,
    ResignFocus,
    Focus,
    EnsureEditorTabActiveVisible,
    FocusSourceControl,
    ShowSettings,
    ShowKeybindings,
    FocusEditor,
    RunPalette(Option<PaletteType>),
    RunPaletteReferences(Vec<EditorLocation>),
    InitPaletteInput(String),
    UpdatePaletteInput(String),
    UpdatePaletteItems(String, Vec<PaletteItem>),
    FilterPaletteItems(String, String, Vec<PaletteItem>),
    UpdateKeymapsFilter(String),
    ResetSettingsFile(String, String),
    UpdateSettingsFile(String, String, Value),
    UpdateSettingsFilter(String),
    FilterKeymaps(String, Arc<Vec<KeyMap>>, Arc<Vec<LapceCommand>>),
    UpdatePickerPwd(PathBuf),
    UpdatePickerItems(PathBuf, HashMap<PathBuf, FileNodeItem>),
    UpdateExplorerItems(PathBuf, HashMap<PathBuf, FileNodeItem>, bool),
    UpdateInstalledPlugins(HashMap<String, PluginDescription>),
    UpdatePluginDescriptions(Vec<PluginDescription>),
    RequestLayout,
    RequestPaint,
    ResetFade,
    //FocusTab,
    CloseTab,
    CloseTabId(WidgetId),
    FocusTabId(WidgetId),
    SwapTab(usize),
    NewTab,
    NextTab,
    PreviousTab,
    FilterItems,
    NewWindow(WindowId),
    ReloadWindow,
    CloseBuffers(Vec<BufferId>),
    RequestPaintRect(Rect),
    ApplyEdits(usize, u64, Vec<TextEdit>),
    ApplyEditsAndSave(usize, u64, Result<Value>),
    DocumentFormat(PathBuf, u64, Result<Value>),
    DocumentFormatAndSave(PathBuf, u64, Result<Value>, Option<WidgetId>),
    DocumentSave(PathBuf, Option<WidgetId>),
    BufferSave(PathBuf, u64, Option<WidgetId>),
    UpdateSemanticStyles(BufferId, PathBuf, u64, Arc<Spans<Style>>),
    UpdateTerminalTitle(TermId, String),
    UpdateHistoryStyle {
        id: BufferId,
        path: PathBuf,
        history: String,
        highlights: Arc<Spans<Style>>,
    },
    UpdateSyntax {
        content: BufferContent,
        rev: u64,
        syntax: SingleUse<Syntax>,
    },
    UpdateHistoryChanges {
        id: BufferId,
        path: PathBuf,
        rev: u64,
        history: String,
        changes: Arc<Vec<DiffLines>>,
    },
    CenterOfWindow,
    UpdateLineChanges(BufferId),
    PublishDiagnostics(PublishDiagnosticsParams),
    WorkDoneProgress(ProgressParams),
    UpdateDiffInfo(DiffInfo),
    EnsureVisible((Rect, (f64, f64), Option<EnsureVisiblePosition>)),
    EnsureRectVisible(Rect),
    EnsureCursorVisible(Option<EnsureVisiblePosition>),
    EnsureCursorPosition(EnsureVisiblePosition),
    EditorViewSize(Size),
    Scroll((f64, f64)),
    ScrollTo((f64, f64)),
    ForceScrollTo(f64, f64),
    SaveAs(BufferContent, PathBuf, WidgetId, bool),
    SaveAsSuccess(BufferContent, u64, PathBuf, WidgetId, bool),
    HomeDir(PathBuf),
    FileChange(notify::Event),
    ProxyUpdateStatus(ProxyStatus),
    CloseTerminal(TermId),
    SplitTerminal(bool, WidgetId),
    SplitTerminalClose(TermId, WidgetId),
    SplitEditor(bool, WidgetId),
    SplitEditorMove(SplitMoveDirection, WidgetId),
    SplitEditorExchange(WidgetId),
    SplitEditorClose(WidgetId),
    Split(bool),
    SplitClose,
    SplitExchange(SplitContent),
    SplitRemove(SplitContent),
    SplitMove(SplitMoveDirection),
    SplitAdd(usize, SplitContent, bool),
    SplitReplace(usize, SplitContent),
    SplitChangeDirection(SplitDirection),
    EditorTabAdd(usize, EditorTabChild),
    EditorTabRemove(usize, bool, bool),
    EditorTabSwap(usize, usize),
    JumpToPosition(Option<WidgetId>, Position),
    JumpToLine(Option<WidgetId>, usize),
    JumpToLocation(Option<WidgetId>, EditorLocation),
    TerminalJumpToLine(i32),
    GoToLocationNew(WidgetId, EditorLocation),
    GotoReference(WidgetId, usize, EditorLocation),
    GotoDefinition(WidgetId, usize, EditorLocation),
    PaletteReferences(usize, Vec<Location>),
    GotoLocation(Location),
    ActiveFileChanged {
        path: Option<PathBuf>,
    },
}
