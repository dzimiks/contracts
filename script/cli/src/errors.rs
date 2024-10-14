use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error as StdError;
use std::fmt;
use std::io::stdout;

// this file is a collection of errors used for the app and a log function to display errors on the main terminal screen. By default the terminal is put into raw mode and displays the alternate screen, so when logging an error the terminal will exit the raw mode and alternate screen, print the error to the terminal and return to the alternate screen.

#[derive(Debug)]
pub struct ConnectionError(pub String);
impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl StdError for ConnectionError {}

#[derive(Debug)]
pub struct UnsupportedExplorerError(pub String, pub String);
impl fmt::Display for UnsupportedExplorerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported explorer: {} ({})", self.0, self.1)
    }
}
impl StdError for UnsupportedExplorerError {}

#[derive(Debug)]
pub struct InvalidEtherscanUrlError(pub String, pub String);
impl fmt::Display for InvalidEtherscanUrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid etherscan url: {} ({})", self.0, self.1)
    }
}
impl StdError for InvalidEtherscanUrlError {}

#[derive(Debug)]
pub struct EtherscanRequestError(pub Box<dyn StdError>);
impl fmt::Display for EtherscanRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Etherscan request error: {}", self.0)
    }
}
impl StdError for EtherscanRequestError {}

#[derive(Debug)]
pub struct EtherscanResponseError(pub String);
impl fmt::Display for EtherscanResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Etherscan response error: {:?}", self.0)
    }
}
impl StdError for EtherscanResponseError {}

pub fn log(message: String) {
    let raw_mode_enabled = crossterm::terminal::is_raw_mode_enabled().unwrap();
    if raw_mode_enabled {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen,);
    }
    println!("{}", message);
    if raw_mode_enabled {
        let _ = execute!(stdout(), EnterAlternateScreen);
        let _ = enable_raw_mode();
    }
}
