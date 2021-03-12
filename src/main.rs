#[macro_use]
extern crate failure;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use failure::{Error, Fallible};
use std::ffi::OsString;
use std::fs::DirBuilder;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use crate::config::Theme;
use crate::font::{FontConfiguration, FontSystemSelection};
use crate::frontend::FrontEndSelection;
use crate::mux::domain::{Domain, LocalDomain};
use crate::mux::Mux;
use crate::pty::cmdbuilder::CommandBuilder;
use crate::pty::PtySize;
use crate::term::color::RgbColor;

mod clipboard;
mod config;
mod core;
mod font;
mod frontend;
mod keyassignment;
mod localtab;
mod mux;
mod pty;
mod ratelim;
mod server;
mod term;
mod window;

#[derive(Debug, Default, Clone)]
struct StartCommand {
    front_end: Option<FrontEndSelection>,

    font_system: Option<FontSystemSelection>,

    /// If true, do not connect to domains marked as connect_automatically
    /// in your miro.toml configuration file.
    no_auto_connect: bool,

    /// Detach from the foreground and become a background process
    daemonize: bool,

    /// Instead of executing your shell, run PROG.
    /// For example: `miro start -- bash -l` will spawn bash
    /// as if it were a login shell.
    prog: Vec<OsString>,
}

pub fn create_user_owned_dirs(p: &Path) -> Fallible<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);

    builder.mode(0o700);

    builder.create(p)?;
    Ok(())
}

fn run_terminal_gui(config: Arc<config::Config>, opts: &StartCommand) -> Result<(), Error> {
    let font_system = config.font_system;
    font_system.set_default();

    let fontconfig = Rc::new(FontConfiguration::new(Arc::clone(&config), font_system));

    let cmd = Some(CommandBuilder::new_default_prog());

    let domain: Arc<dyn Domain> = Arc::new(LocalDomain::new("local", &config)?);
    let mux = Rc::new(mux::Mux::new(&config, Some(domain.clone())));
    Mux::set_mux(&mux);

    let front_end = opts.front_end.unwrap_or(config.front_end);
    let gui = front_end.try_new()?;
    domain.attach()?;

    if mux.is_empty() {
        let window_id = mux.new_empty_window();
        let tab = mux.default_domain().spawn(PtySize::default(), cmd, window_id)?;
        gui.spawn_new_window(mux.config(), &fontconfig, &tab, window_id)?;
    }

    gui.run_forever()
}

fn run(theme: Option<Theme>) -> Result<(), Error> {
    let config = Arc::new(config::Config::default_config(theme));

    let start = StartCommand::default();
    run_terminal_gui(config, &start)
}

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::with_name("theme")
                .short("t")
                .long("theme")
                .help("Which theme to use.")
                .possible_values(&["mario", "sonic", "pika", "mega", "kirby"])
                .takes_value(true),
        )
        .get_matches();

    let theme = match matches.value_of("theme") {
        Some("sonic") => Some(Theme {
            spritesheet_path: String::from("assets/sonic.json"),
            header_color: RgbColor { red: 8, green: 129, blue: 0 },
        }),
        Some("pika") => Some(Theme {
            spritesheet_path: String::from("assets/pika.json"),
            header_color: RgbColor { red: 176, green: 139, blue: 24 },
        }),
        Some("mega") => Some(Theme {
            spritesheet_path: String::from("assets/mega.json"),
            header_color: RgbColor { red: 1, green: 135, blue: 147 },
        }),
        Some("kirby") => Some(Theme {
            spritesheet_path: String::from("assets/kirby.json"),
            header_color: RgbColor { red: 242, green: 120, blue: 141 },
        }),
        _ => None,
    };

    run(theme)
}
