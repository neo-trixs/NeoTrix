macro_rules! define_plugin {
    ($name:ident, $plugin_name:expr, $desc:expr, $($action:expr => $handler:expr),* $(,)?) => {
        pub struct $name;

        impl $crate::domain::DomainPlugin for $name {
            fn name(&self) -> &str { $plugin_name }
            fn description(&self) -> &str { $desc }

            fn actions(&self) -> Vec<$crate::domain::ActionSpec> {
                vec![$(
                    $crate::domain::ActionSpec {
                        name: $action.into(),
                        description: String::new(),
                        params: vec![],
                        returns: "Value".into(),
                    }
                ),*]
            }

            fn call(&self, action: &str, args: $crate::domain::serde_json::Value)
                -> Result<$crate::domain::serde_json::Value, $crate::domain::DomainError>
            {
                match action {
                    $($action => $handler(args),)*
                    _ => Err($crate::domain::DomainError {
                        code: "UNKNOWN_ACTION".into(),
                        message: format!("Unknown action: {}", action),
                        recoverable: true,
                    })
                }
            }
        }
    };
}

pub(crate) use define_plugin;
