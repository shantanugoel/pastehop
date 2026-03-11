use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookAction {
    InjectText,
    PassthroughKey,
    Noop,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookResponse {
    pub action: HookAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HookResponse {
    pub fn inject_text(text: impl Into<String>) -> Self {
        Self {
            action: HookAction::InjectText,
            text: Some(text.into()),
            message: None,
        }
    }

    pub fn passthrough_key() -> Self {
        Self {
            action: HookAction::PassthroughKey,
            text: None,
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            action: HookAction::Error,
            text: None,
            message: Some(message.into()),
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{HookAction, HookResponse};

    #[test]
    fn serializes_inject_text() {
        let payload = HookResponse::inject_text("/remote/file.png");
        let encoded = payload.to_json().expect("hook response should serialize");

        assert_eq!(
            encoded,
            "{\"action\":\"inject_text\",\"text\":\"/remote/file.png\"}"
        );
    }

    #[test]
    fn serializes_passthrough() {
        let payload = HookResponse::passthrough_key();
        let encoded = payload.to_json().expect("hook response should serialize");

        assert_eq!(encoded, "{\"action\":\"passthrough_key\"}");
    }

    #[test]
    fn round_trips_error_message() {
        let payload = HookResponse::error("upload failed");
        let encoded = payload.to_json().expect("hook response should serialize");
        let decoded: HookResponse =
            serde_json::from_str(&encoded).expect("hook response should deserialize");

        assert_eq!(decoded.action, HookAction::Error);
        assert_eq!(decoded.message.as_deref(), Some("upload failed"));
    }

    #[test]
    fn round_trips_noop_message() {
        let payload = HookResponse {
            action: HookAction::Noop,
            text: None,
            message: Some("noop".to_owned()),
        };
        let encoded = payload.to_json().expect("hook response should serialize");
        let decoded: HookResponse =
            serde_json::from_str(&encoded).expect("hook response should deserialize");

        assert_eq!(decoded.action, HookAction::Noop);
        assert_eq!(decoded.message.as_deref(), Some("noop"));
    }
}
