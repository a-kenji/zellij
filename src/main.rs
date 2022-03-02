mod commands;
mod install;
mod sessions;
#[cfg(test)]
mod tests;

use zellij_utils::{
    clap::Parser,
    cli::{CliArgs, Command, Sessions},
    logging::*,
};
use knuffel;

fn main() {
    configure_logger();
    let opts = CliArgs::parse();

    if let Some(Command::Sessions(Sessions::ListSessions)) = opts.command {
        commands::list_sessions();
    } 
    if let Some(Command::Sessions(Sessions::Action{ action })) = opts.command {
        println!("{:?}", action);
        if let Some(action) = action {
        let parsed: zellij_utils::input::actions::ActionsFromKdl =  knuffel::parse("", &action).unwrap();
        println!("{:?}", parsed);
        }
    } 
        else if let Some(Command::Sessions(Sessions::KillAllSessions { yes })) = opts.command {
        commands::kill_all_sessions(yes);
    } else if let Some(Command::Sessions(Sessions::KillSession { ref target_session })) =
        opts.command
    {
        commands::kill_session(target_session);
    } else if let Some(path) = opts.server {
        commands::start_server(path);
    } else {
        commands::start_client(opts);
    }
}
