#[derive(Debug, Clone, PartialEq)]
pub enum VoiceCommand {
    OpenSettings,
    RunCommand(String),
    SwitchSession(String),
    ShowHelp,
    WakeTrigger,
    Unknown(String),
}

impl VoiceCommand {
    pub fn parse(text: &str) -> Self {
        let text = text.trim().to_lowercase();
        let text = text.trim_matches(|c: char| c.is_ascii_punctuation());

        if text.contains("open settings") || text.contains("open setting") {
            return VoiceCommand::OpenSettings;
        }
        if text.contains("show help") || text == "help" {
            return VoiceCommand::ShowHelp;
        }
        if let Some(cmd) = text.strip_prefix("run command ") {
            let cmd = cmd.trim().to_string();
            if !cmd.is_empty() {
                return VoiceCommand::RunCommand(cmd);
            }
        }
        if let Some(name) = text.strip_prefix("switch session ") {
            let name = name.trim().to_string();
            if !name.is_empty() {
                return VoiceCommand::SwitchSession(name);
            }
        }
        if let Some(name) = text.strip_prefix("switch to session ") {
            let name = name.trim().to_string();
            if !name.is_empty() {
                return VoiceCommand::SwitchSession(name);
            }
        }
        if text.contains("run ") {
            let cmd = text.split("run ").nth(1).unwrap_or("").trim().to_string();
            if !cmd.is_empty() {
                return VoiceCommand::RunCommand(cmd);
            }
        }
        VoiceCommand::Unknown(text.to_string())
    }

    pub fn description(&self) -> &str {
        match self {
            VoiceCommand::OpenSettings => "open settings panel",
            VoiceCommand::RunCommand(_) => "execute a command",
            VoiceCommand::SwitchSession(_) => "switch to another session",
            VoiceCommand::ShowHelp => "show help information",
            VoiceCommand::WakeTrigger => "wake word detected",
            VoiceCommand::Unknown(_) => "unrecognized nt_act_voice command",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_open_settings() {
        assert_eq!(
            VoiceCommand::parse("open settings"),
            VoiceCommand::OpenSettings
        );
        assert_eq!(
            VoiceCommand::parse("Open Settings!"),
            VoiceCommand::OpenSettings
        );
        assert_eq!(
            VoiceCommand::parse("please open settings"),
            VoiceCommand::OpenSettings
        );
    }

    #[test]
    fn test_parse_run_command() {
        assert_eq!(
            VoiceCommand::parse("run command deploy"),
            VoiceCommand::RunCommand("deploy".into())
        );
        assert_eq!(
            VoiceCommand::parse("run cargo check"),
            VoiceCommand::RunCommand("cargo check".into())
        );
    }

    #[test]
    fn test_parse_switch_session() {
        assert_eq!(
            VoiceCommand::parse("switch session work"),
            VoiceCommand::SwitchSession("work".into())
        );
        assert_eq!(
            VoiceCommand::parse("switch to session gaming"),
            VoiceCommand::SwitchSession("gaming".into())
        );
    }

    #[test]
    fn test_parse_show_help() {
        assert_eq!(VoiceCommand::parse("show help"), VoiceCommand::ShowHelp);
        assert_eq!(VoiceCommand::parse("help"), VoiceCommand::ShowHelp);
        assert_eq!(VoiceCommand::parse("Show Help!"), VoiceCommand::ShowHelp);
    }

    #[test]
    fn test_parse_unknown() {
        let result = VoiceCommand::parse("what is the weather");
        assert!(matches!(result, VoiceCommand::Unknown(_)));
    }

    #[test]
    fn test_description_non_empty() {
        let cmds = vec![
            VoiceCommand::OpenSettings,
            VoiceCommand::RunCommand("test".into()),
            VoiceCommand::SwitchSession("test".into()),
            VoiceCommand::ShowHelp,
            VoiceCommand::WakeTrigger,
            VoiceCommand::Unknown("blah".into()),
        ];
        for cmd in cmds {
            assert!(!cmd.description().is_empty());
        }
    }

    #[test]
    fn test_parse_run_without_command_keyword() {
        let result = VoiceCommand::parse("run tests");
        assert_eq!(result, VoiceCommand::RunCommand("tests".into()));
    }
}
