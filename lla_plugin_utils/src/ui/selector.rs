#[cfg(feature = "interactive")]
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

pub struct InteractiveSelector;

impl InteractiveSelector {
    #[cfg(feature = "interactive")]
    pub fn select_one<T: ToString>(
        items: &[T],
        prompt: &str,
        default: Option<usize>,
    ) -> Result<Option<usize>, String> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&items.iter().map(|i| i.to_string()).collect::<Vec<String>>())
            .default(default.unwrap_or(0))
            .interact_opt()
            .map_err(|e| format!("Failed to show selector: {}", e))
    }

    #[cfg(not(feature = "interactive"))]
    pub fn select_one<T: ToString>(
        _items: &[T],
        _prompt: &str,
        _default: Option<usize>,
    ) -> Result<Option<usize>, String> {
        Err("Interactive features are not enabled".to_string())
    }

    #[cfg(feature = "interactive")]
    pub fn select_multiple<T: ToString>(
        items: &[T],
        prompt: &str,
        defaults: Option<&[bool]>,
    ) -> Result<Vec<usize>, String> {
        MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&items.iter().map(|i| i.to_string()).collect::<Vec<String>>())
            .defaults(defaults.unwrap_or(&vec![false; items.len()]))
            .interact()
            .map_err(|e| format!("Failed to show selector: {}", e))
    }

    #[cfg(not(feature = "interactive"))]
    pub fn select_multiple<T: ToString>(
        _items: &[T],
        _prompt: &str,
        _defaults: Option<&[bool]>,
    ) -> Result<Vec<usize>, String> {
        Err("Interactive features are not enabled".to_string())
    }

    #[cfg(feature = "interactive")]
    pub fn confirm(prompt: &str, default: bool) -> Result<bool, String> {
        dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| format!("Failed to show prompt: {}", e))
    }

    #[cfg(not(feature = "interactive"))]
    pub fn confirm(_prompt: &str, _default: bool) -> Result<bool, String> {
        Err("Interactive features are not enabled".to_string())
    }

    #[cfg(feature = "interactive")]
    pub fn input(prompt: &str) -> Result<String, String> {
        dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact_text()
            .map_err(|e| format!("Failed to get input: {}", e))
    }

    #[cfg(not(feature = "interactive"))]
    pub fn input(_prompt: &str) -> Result<String, String> {
        Err("Interactive features are not enabled".to_string())
    }

    #[cfg(feature = "interactive")]
    pub fn select_with_custom<T: ToString>(
        items: &[T],
        prompt: &str,
        custom_prompt: &str,
    ) -> Result<Option<String>, String> {
        let mut display_items = items.iter().map(|i| i.to_string()).collect::<Vec<String>>();
        display_items.push("(Custom)".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&display_items)
            .default(0)
            .interact()
            .map_err(|e| format!("Failed to show selector: {}", e))?;

        if selection == items.len() {
            dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(custom_prompt)
                .interact_text()
                .map(Some)
                .map_err(|e| format!("Failed to get input: {}", e))
        } else {
            Ok(Some(items[selection].to_string()))
        }
    }

    #[cfg(not(feature = "interactive"))]
    pub fn select_with_custom<T: ToString>(
        _items: &[T],
        _prompt: &str,
        _custom_prompt: &str,
    ) -> Result<Option<String>, String> {
        Err("Interactive features are not enabled".to_string())
    }
}
