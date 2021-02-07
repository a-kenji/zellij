use super::actions::Action;
use super::keybinds::get_default_keybinds;
use crate::common::{update_state, AppInstruction, AppState, SenderWithContext, OPENCALLS};
/// Module for handling input
use crate::errors::ContextType;
use crate::os_input_output::OsApi;
use crate::pty_bus::PtyInstruction;
use crate::screen::ScreenInstruction;
use crate::wasm_vm::PluginInstruction;
use crate::CommandIsExecuting;

use strum_macros::EnumIter;
use termion::input::TermReadEventsAndRaw;

use super::keybinds::key_to_action;

struct InputHandler {
    mode: InputMode,
    mode_is_persistent: bool,
    os_input: Box<dyn OsApi>,
    command_is_executing: CommandIsExecuting,
    send_screen_instructions: SenderWithContext<ScreenInstruction>,
    send_pty_instructions: SenderWithContext<PtyInstruction>,
    send_plugin_instructions: SenderWithContext<PluginInstruction>,
    send_app_instructions: SenderWithContext<AppInstruction>,
}

impl InputHandler {
    fn new(
        os_input: Box<dyn OsApi>,
        command_is_executing: CommandIsExecuting,
        send_screen_instructions: SenderWithContext<ScreenInstruction>,
        send_pty_instructions: SenderWithContext<PtyInstruction>,
        send_plugin_instructions: SenderWithContext<PluginInstruction>,
        send_app_instructions: SenderWithContext<AppInstruction>,
    ) -> Self {
        InputHandler {
            mode: InputMode::Normal,
            mode_is_persistent: false,
            os_input,
            command_is_executing,
            send_screen_instructions,
            send_pty_instructions,
            send_plugin_instructions,
            send_app_instructions,
        }
    }

    /// Main event loop
    fn get_input(&mut self) {
        let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
        err_ctx.add_call(ContextType::StdinHandler);
        self.send_pty_instructions.update(err_ctx);
        self.send_app_instructions.update(err_ctx);
        self.send_screen_instructions.update(err_ctx);
        if let Ok(keybinds) = get_default_keybinds() {
            'input_loop: loop {
                let entry_mode = self.mode;
                //@@@ I think this should actually just iterate over stdin directly
                let stdin_buffer = self.os_input.read_from_stdin();
                drop(
                    self.send_plugin_instructions
                        .send(PluginInstruction::GlobalInput(stdin_buffer.clone())),
                );
                for key_result in stdin_buffer.events_and_raw() {
                    match key_result {
                        Ok((event, raw_bytes)) => match event {
                            termion::event::Event::Key(key) => {
                                let should_break = self.dispatch_action(key_to_action(
                                    &key, raw_bytes, &self.mode, &keybinds,
                                ));
                                //@@@ This is a hack until we dispatch more than one action per key stroke
//                                if entry_mode == InputMode::Command
//                                    && self.mode == InputMode::Command
                                if entry_mode == self.mode && !self.mode_is_persistent {
                                    self.mode = InputMode::Normal;
                                    update_state(&self.send_app_instructions, |_| AppState {
                                        input_mode: self.mode,
                                    });
                                }
                                if should_break {
                                    break 'input_loop;
                                }
                            }
                            termion::event::Event::Mouse(_)
                            | termion::event::Event::Unsupported(_) => {
                                unimplemented!("Mouse and unsupported events aren't supported!");
                            }
                        },
                        Err(err) => panic!("Encountered read error: {:?}", err),
                    }
                }
            }
        } else {
            //@@@ Error handling?
            self.exit();
        }
    }

    fn dispatch_action(&mut self, action: Action) -> bool {
        let mut interrupt_loop = false;

        match action {
            Action::Write(val) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ClearScroll)
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::WriteCharacter(val))
                    .unwrap();
            }
            Action::Quit => {
                self.exit();
                interrupt_loop = true;
            }
            Action::SwitchToMode(mode) => {
                self.mode = mode;
                if mode == InputMode::Normal {
                    self.mode_is_persistent = false;
                }
                update_state(&self.send_app_instructions, |_| AppState {
                    input_mode: self.mode,
                });
                self.send_screen_instructions
                    .send(ScreenInstruction::Render)
                    .unwrap();
            }
            Action::TogglePersistentMode => {
                self.mode_is_persistent = !self.mode_is_persistent;
            }
            Action::Resize(direction) => {
                let screen_instr = match direction {
                    super::actions::Direction::Left => ScreenInstruction::ResizeLeft,
                    super::actions::Direction::Right => ScreenInstruction::ResizeRight,
                    super::actions::Direction::Up => ScreenInstruction::ResizeUp,
                    super::actions::Direction::Down => ScreenInstruction::ResizeDown,
                };
                self.send_screen_instructions.send(screen_instr).unwrap();
            }
            Action::SwitchFocus(_) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::MoveFocus)
                    .unwrap();
            }
            Action::MoveFocus(direction) => {
                let screen_instr = match direction {
                    super::actions::Direction::Left => ScreenInstruction::MoveFocusLeft,
                    super::actions::Direction::Right => ScreenInstruction::MoveFocusRight,
                    super::actions::Direction::Up => ScreenInstruction::MoveFocusUp,
                    super::actions::Direction::Down => ScreenInstruction::MoveFocusDown,
                };
                self.send_screen_instructions.send(screen_instr).unwrap();
            }
            Action::ScrollUp => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ScrollUp)
                    .unwrap();
            }
            Action::ScrollDown => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ScrollDown)
                    .unwrap();
            }
            Action::ToggleFocusFullscreen => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ToggleActiveTerminalFullscreen)
                    .unwrap();
            }
            Action::NewPane(direction) => {
                let pty_instr = match direction {
                    Some(super::actions::Direction::Left) => {
                        PtyInstruction::SpawnTerminalVertically(None)
                    }
                    Some(super::actions::Direction::Right) => {
                        PtyInstruction::SpawnTerminalVertically(None)
                    }
                    Some(super::actions::Direction::Up) => {
                        PtyInstruction::SpawnTerminalHorizontally(None)
                    }
                    Some(super::actions::Direction::Down) => {
                        PtyInstruction::SpawnTerminalHorizontally(None)
                    }
                    // No direction specified - try to put it in the biggest available spot
                    None => PtyInstruction::SpawnTerminal(None),
                };
                self.command_is_executing.opening_new_pane();
                self.send_pty_instructions.send(pty_instr).unwrap();
                self.command_is_executing.wait_until_new_pane_is_opened();
            }
            Action::CloseFocus => {
                self.command_is_executing.closing_pane();
                self.send_screen_instructions
                    .send(ScreenInstruction::CloseFocusedPane)
                    .unwrap();
                self.command_is_executing.wait_until_pane_is_closed();
            }
            Action::NewTab => {
                self.command_is_executing.opening_new_pane();
                self.send_pty_instructions
                    .send(PtyInstruction::NewTab)
                    .unwrap();
                self.command_is_executing.wait_until_new_pane_is_opened();
            }
            Action::GoToNextTab => {
                self.send_screen_instructions
                    .send(ScreenInstruction::SwitchTabNext)
                    .unwrap();
            }
            Action::GoToPreviousTab => {
                self.send_screen_instructions
                    .send(ScreenInstruction::SwitchTabPrev)
                    .unwrap();
            }
            Action::CloseTab => {
                self.command_is_executing.closing_pane();
                self.send_screen_instructions
                    .send(ScreenInstruction::CloseTab)
                    .unwrap();
                self.command_is_executing.wait_until_pane_is_closed();
            }
        }

        interrupt_loop
    }

    /// Routine to be called when the input handler exits (at the moment this is the
    /// same as quitting mosaic)
    fn exit(&mut self) {
        self.send_app_instructions
            .send(AppInstruction::Exit)
            .unwrap();
    }
}

/// Dictates whether we're in command mode, persistent command mode, normal mode or exiting:
/// - Normal mode either writes characters to the terminal, or switches to command mode
///   using a particular key control
/// - Command mode intercepts characters to control mosaic itself, before switching immediately
///   back to normal mode
/// - Persistent command mode is the same as command mode, but doesn't return automatically to
///   normal mode
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, EnumIter)]
pub enum InputMode {
    Normal,
    Command,
    CommandPersistent,
    Resize,
    Pane,
    Tab,
    Scroll,
    Exiting,
}

// FIXME: This should be auto-generated from the soon-to-be-added `get_default_keybinds`
pub fn get_help(mode: &InputMode) -> Vec<String> {
    let command_help = vec![
        "<r> Resize mode".into(),
        "<p> Pane mode".into(),
        "<t> Tab mode".into(),
        // "<n/b/z> Split".into(),
        // "<p> Focus Next".into(),
        // "<x> Close Pane".into(),
        "<q> Quit".into(),
        // "<PgUp/PgDown> Scroll".into(),
        // "<1> New Tab".into(),
        // "<2/3> Move Tab".into(),
        // "<4> Close Tab".into(),
    ];
    match mode {
        InputMode::Normal => vec!["<Ctrl-g> Command Mode".into()],
        InputMode::Command => [
            vec![
                "<Ctrl-g> Persistent Mode".into(),
                "<ESC> Normal Mode".into(),
            ],
            command_help,
        ]
        .concat(),
        InputMode::CommandPersistent => {
            [vec!["<ESC/Ctrl-g> Normal Mode".into()], command_help].concat()
        }
        InputMode::Exiting => vec!["Bye from Mosaic!".into()],
        InputMode::Resize => vec![
            "<arrows/hjkl/ctrl+bnpf> resize current pane".into(),
            "<ESC> Normal Mode".into(),
            "<q> Quit".into(),
        ],
        InputMode::Pane => vec![
            "<arrows/hjkl/ctrl+bnpf> move focus".into(),
            "<p> next pane".into(),
            "<n> new pane".into(),
            "<d> down split".into(),
            "<r> right split".into(),
            "<x> exit pane".into(),
            "<f> fullscreen pane".into(),
            "<ESC> Normal Mode".into(),
            "<q> Quit".into(),
        ],
        InputMode::Tab => vec![
            "<arrows/hjkl/ctrl+bnpf> next/prev tab".into(),
            "<n> new tab".into(),
            "<x> exit tab".into(),
            "<ESC> Normal Mode".into(),
            "<q> Quit".into(),
        ],
        InputMode::Scroll => vec![
            "<arrows/jk/ctrl+np> scroll up/down".into(),
            "<ESC> Normal Mode".into(),
            "<q> Quit".into(),
        ],
    }
}

/// Entry point to the module that instantiates a new InputHandler and calls its
/// reading loop
pub fn input_loop(
    os_input: Box<dyn OsApi>,
    command_is_executing: CommandIsExecuting,
    send_screen_instructions: SenderWithContext<ScreenInstruction>,
    send_pty_instructions: SenderWithContext<PtyInstruction>,
    send_plugin_instructions: SenderWithContext<PluginInstruction>,
    send_app_instructions: SenderWithContext<AppInstruction>,
) {
    let _handler = InputHandler::new(
        os_input,
        command_is_executing,
        send_screen_instructions,
        send_pty_instructions,
        send_plugin_instructions,
        send_app_instructions,
    )
    .get_input();
}