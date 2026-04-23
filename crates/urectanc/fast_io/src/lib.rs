mod input;
mod output;

#[cfg(unix)]
pub use input::Input;
pub use output::Output;

pub fn stdin() -> Input {
    Input::stdin()
}

pub fn stdout() -> Output<std::io::StdoutLock<'static>> {
    Output::stdout()
}
