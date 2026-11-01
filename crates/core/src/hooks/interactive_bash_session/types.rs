use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveBashSessionState {
    pub session_id: String,
    #[serde(serialize_with = "serialize_hashset", deserialize_with = "deserialize_hashset")]
    pub tmux_sessions: HashSet<String>,
    pub updated_at: u64, // timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedInteractiveBashSessionState {
    pub session_id: String,
    pub tmux_sessions: Vec<String>,
    pub updated_at: u64, // timestamp in milliseconds
}

// ฟังก์ชันช่วย serialize/deserialize HashSet เป็น Vec
fn serialize_hashset<S>(hash_set: &HashSet<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let vec: Vec<String> = hash_set.iter().cloned().collect();
    vec.serialize(serializer)
}

fn deserialize_hashset<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}

impl From<SerializedInteractiveBashSessionState> for InteractiveBashSessionState {
    fn from(serialized: SerializedInteractiveBashSessionState) -> Self {
        let mut tmux_sessions = HashSet::new();
        for session in serialized.tmux_sessions {
            tmux_sessions.insert(session);
        }
        
        Self {
            session_id: serialized.session_id,
            tmux_sessions,
            updated_at: serialized.updated_at,
        }
    }
}

impl From<InteractiveBashSessionState> for SerializedInteractiveBashSessionState {
    fn from(state: InteractiveBashSessionState) -> Self {
        let tmux_sessions = state.tmux_sessions.into_iter().collect();
        
        Self {
            session_id: state.session_id,
            tmux_sessions,
            updated_at: state.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_between_serialized_and_unserialized() {
        let mut tmux_sessions = HashSet::new();
        tmux_sessions.insert("session1".to_string());
        tmux_sessions.insert("session2".to_string());
        
        let state = InteractiveBashSessionState {
            session_id: "test_session".to_string(),
            tmux_sessions,
            updated_at: 1234567890,
        };
        
        let serialized: SerializedInteractiveBashSessionState = state.clone().into();
        let deserialized: InteractiveBashSessionState = serialized.into();
        
        assert_eq!(state.session_id, deserialized.session_id);
        assert_eq!(state.updated_at, deserialized.updated_at);
        assert_eq!(state.tmux_sessions, deserialized.tmux_sessions);
    }
}