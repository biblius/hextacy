#[cfg(test)]
mod tests {
    use actors::Signal;
    use serde::{Deserialize, Serialize};
    use websocket::signals::ws_signal::WsSignal;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SampleData {
        lol: String,
        lel: String,
    }

    #[test]
    fn test_from_ws() {
        let data = SampleData {
            lol: "lol".to_string(),
            lel: "lel".to_string(),
        };
        let ws_signal = WsSignal::new("SampleData", None, data.clone());

        let signal: Signal<SampleData> = ws_signal.into();

        assert_eq!(signal.data().lol, "lol");
        assert_eq!(signal.data().lel, "lel");

        assert!(signal.to().is_none());
    }
}
