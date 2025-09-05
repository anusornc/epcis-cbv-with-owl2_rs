use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EpcisEvent {
    pub event_id: String,
    pub event_type: String,
    pub event_time: String,
    pub record_time: String,
    pub event_action: String,
    pub epc_list: Vec<String>,
    pub biz_step: Option<String>,
    pub disposition: Option<String>,
    pub biz_location: Option<String>,
}

impl Default for EpcisEvent {
    fn default() -> Self {
        Self {
            event_id: String::new(),
            event_type: "ObjectEvent".to_string(),
            event_time: chrono::Utc::now().to_rfc3339(),
            record_time: chrono::Utc::now().to_rfc3339(),
            event_action: "ADD".to_string(),
            epc_list: Vec::new(),
            biz_step: None,
            disposition: None,
            biz_location: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epcis_event_default() {
        let event = EpcisEvent::default();
        assert_eq!(event.event_id, "");
        assert_eq!(event.event_type, "ObjectEvent");
        assert_eq!(event.event_action, "ADD");
        assert!(event.epc_list.is_empty());
        assert!(event.biz_step.is_none());
        assert!(event.disposition.is_none());
        assert!(event.biz_location.is_none());
    }

    #[test]
    fn test_epcis_event_creation() {
        let event = EpcisEvent {
            event_id: "test-001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };

        assert_eq!(event.event_id, "test-001");
        assert_eq!(event.event_type, "ObjectEvent");
        assert_eq!(event.event_action, "ADD");
        assert_eq!(event.epc_list.len(), 1);
        assert_eq!(event.biz_step, Some("commissioning".to_string()));
        assert_eq!(event.disposition, Some("active".to_string()));
        assert_eq!(event.biz_location, Some("urn:epc:id:sgln:123456.789.0".to_string()));
    }

    #[test]
    fn test_epcis_event_serialization() {
        let event = EpcisEvent {
            event_id: "test-001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: EpcisEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_epcis_event_partial_fields() {
        let event = EpcisEvent {
            event_id: "minimal-event".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "OBSERVE".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: None,
            disposition: None,
            biz_location: None,
        };

        assert_eq!(event.event_id, "minimal-event");
        assert_eq!(event.event_action, "OBSERVE");
        assert!(event.biz_step.is_none());
        assert!(event.disposition.is_none());
        assert!(event.biz_location.is_none());
    }

    #[test]
    fn test_epcis_event_multiple_epcs() {
        let event = EpcisEvent {
            event_id: "multi-epc-event".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec![
                "urn:epc:id:sgtin:123456.789.100".to_string(),
                "urn:epc:id:sgtin:123456.789.101".to_string(),
                "urn:epc:id:sgtin:123456.789.102".to_string(),
            ],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };

        assert_eq!(event.epc_list.len(), 3);
        assert!(event.epc_list.contains(&"urn:epc:id:sgtin:123456.789.100".to_string()));
        assert!(event.epc_list.contains(&"urn:epc:id:sgtin:123456.789.101".to_string()));
        assert!(event.epc_list.contains(&"urn:epc:id:sgtin:123456.789.102".to_string()));
    }
}