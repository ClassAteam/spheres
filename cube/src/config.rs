#[derive(Debug, Clone)]
pub struct AppConfig {
    pub debug_ui_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            debug_ui_enabled: true,
        }
    }
}
