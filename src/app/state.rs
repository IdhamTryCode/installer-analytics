#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Confirmation,
    EnvSetup,
    ConfigSelection,
    Installing,
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuSelection {
    Proceed,
    GenerateEnv,
    GenerateConfig,
    Cancel,
}
