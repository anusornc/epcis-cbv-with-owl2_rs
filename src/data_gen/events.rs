use serde::{Serialize, Deserialize};
use chrono::Utc;
use uuid::Uuid;
use crate::data_gen::entities::{Location, Product, BusinessEntity};

/// EPCIS Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    ObjectEvent,
    AggregationEvent,
    QuantityEvent,
    TransactionEvent,
    TransformationEvent,
}

/// EPCIS Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpcisEvent {
    pub uri: String,
    pub event_type: EventType,
    pub event_time: String,
    pub record_time: String,
    pub event_id: String,
    pub action: String,
    pub biz_step: String,
    pub disposition: String,
    pub epc_list: Vec<String>,
    pub read_point: Option<String>,
    pub biz_location: Option<String>,
    pub quantity: Option<u32>,
    pub business_transaction_list: Vec<BusinessTransaction>,
}

/// Business transaction reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessTransaction {
    pub transaction_type: String,
    pub transaction_id: String,
}

/// Supply chain journey step
#[derive(Debug, Clone)]
pub struct JourneyStep {
    pub from_location: String,
    pub to_location: String,
    pub event_type: EventType,
    pub biz_step: String,
    pub estimated_duration_hours: u32,
}

/// Generator for EPCIS events
pub struct EventGenerator {
    business_steps: Vec<String>,
    dispositions: Vec<String>,
    actions: Vec<String>,
}

impl EventGenerator {
    pub fn new() -> Self {
        Self {
            business_steps: vec![
                "urn:epcglobal:cbv:bizstep:commissioning".to_string(),
                "urn:epcglobal:cbv:bizstep:encoding".to_string(),
                "urn:epcglobal:cbv:bizstep:manufacturing".to_string(),
                "urn:epcglobal:cbv:bizstep:testing".to_string(),
                "urn:epcglobal:cbv:bizstep:quality_control".to_string(),
                "urn:epcglobal:cbv:bizstep:packing".to_string(),
                "urn:epcglobal:cbv:bizstep:shipping".to_string(),
                "urn:epcglobal:cbv:bizstep:receiving".to_string(),
                "urn:epcglobal:cbv:bizstep:storing".to_string(),
                "urn:epcglobal:cbv:bizstep:inventory_check".to_string(),
                "urn:epcglobal:cbv:bizstep:pricing".to_string(),
                "urn:epcglobal:cbv:bizstep:displaying".to_string(),
                "urn:epcglobal:cbv:bizstep:selling".to_string(),
                "urn:epcglobal:cbv:bizstep:customer_pickup".to_string(),
            ],
            dispositions: vec![
                "urn:epcglobal:cbv:disp:in_progress".to_string(),
                "urn:epcglobal:cbv:disp:active".to_string(),
                "urn:epcglobal:cbv:disp:inactive".to_string(),
                "urn:epcglobal:cbv:disp:expired".to_string(),
                "urn:epcglobal:cbv:disp:damaged".to_string(),
                "urn:epcglobal:cbv:disp:inspected".to_string(),
                "urn:epcglobal:cbv:disp:certified".to_string(),
                "urn:epcglobal:cbv:disp:in_transit".to_string(),
                "urn:epcglobal:cbv:disp:owned".to_string(),
                "urn:epcglobal:cbv:disp:consigned".to_string(),
            ],
            actions: vec![
                "ADD".to_string(),
                "OBSERVE".to_string(),
                "DELETE".to_string(),
            ],
        }
    }
    
    /// Generate supply chain events for products across locations
    pub fn generate_supply_chain_events(
        &self,
        products: &[Product],
        locations: &[Location],
        _business_entities: &[BusinessEntity],
        count: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        
        // Generate manufacturing events
        let manufacturing_events = self.generate_manufacturing_events(products, locations, count / 4)?;
        events.extend(manufacturing_events);
        
        // Generate logistics events
        let logistics_events = self.generate_logistics_events(products, locations, count / 3)?;
        events.extend(logistics_events);
        
        // Generate retail events
        let retail_events = self.generate_retail_events(products, locations, count / 3)?;
        events.extend(retail_events);
        
        // Generate quality control events
        let qc_events = self.generate_quality_control_events(products, locations, count / 12)?;
        events.extend(qc_events);
        
        Ok(events)
    }
    
    /// Generate manufacturing events
    fn generate_manufacturing_events(
        &self,
        products: &[Product],
        locations: &[Location],
        count: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        let factories: Vec<&Location> = locations.iter().filter(|l| l.location_type == "Factory").collect();
        
        if factories.is_empty() {
            return Ok(events);
        }
        
        for i in 0..count {
            let product = &products[i % products.len()];
            let factory = &factories[i % factories.len()];
            
            let event_time = Utc::now() - chrono::Duration::days((count - i) as i64);
            
            // Manufacturing process sequence
            let biz_step = match i % 5 {
                0 => "urn:epcglobal:cbv:bizstep:manufacturing",
                1 => "urn:epcglobal:cbv:bizstep:testing",
                2 => "urn:epcglobal:cbv:bizstep:quality_control",
                3 => "urn:epcglobal:cbv:bizstep:commissioning",
                4 => "urn:epcglobal:cbv:bizstep:encoding",
                _ => "urn:epcglobal:cbv:bizstep:manufacturing",
            };
            
            events.push(EpcisEvent {
                uri: format!("http://example.com/event/manufacturing/{}", Uuid::new_v4()),
                event_type: EventType::ObjectEvent,
                event_time: event_time.to_rfc3339(),
                record_time: (event_time + chrono::Duration::minutes(5)).to_rfc3339(),
                event_id: format!("MANUF-{:08}", i + 1),
                action: "ADD".to_string(),
                biz_step: biz_step.to_string(),
                disposition: "urn:epcglobal:cbv:disp:in_progress".to_string(),
                epc_list: vec![product.epc.clone()],
                read_point: Some(format!("{}/line{}", factory.uri, i % 10 + 1)),
                biz_location: Some(factory.uri.clone()),
                quantity: Some(1),
                business_transaction_list: vec![],
            });
        }
        
        Ok(events)
    }
    
    /// Generate logistics events (shipping, receiving, storing)
    fn generate_logistics_events(
        &self,
        products: &[Product],
        locations: &[Location],
        count: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        let warehouses: Vec<&Location> = locations.iter().filter(|l| l.location_type == "Warehouse").collect();
        let distribution_centers: Vec<&Location> = locations.iter()
            .filter(|l| l.location_type == "DistributionCenter").collect();
        
        for i in 0..count {
            let product = &products[i % products.len()];
            let event_time = Utc::now() - chrono::Duration::days((count / 2 - i) as i64);
            
            // Create realistic logistics flow
            let (from_location, to_location, biz_step) = if i % 3 == 0 && !warehouses.is_empty() {
                // Factory to warehouse
                let factory = locations.iter().find(|l| l.location_type == "Factory").unwrap();
                let warehouse = &warehouses[i % warehouses.len()];
                (factory.uri.clone(), warehouse.uri.clone(), "urn:epcglobal:cbv:bizstep:shipping")
            } else if i % 3 == 1 && !distribution_centers.is_empty() {
                // Warehouse to distribution center
                let warehouse = &warehouses[i % warehouses.len()];
                let dc = &distribution_centers[i % distribution_centers.len()];
                (warehouse.uri.clone(), dc.uri.clone(), "urn:epcglobal:cbv:bizstep:transporting")
            } else {
                // Distribution center to retail
                let dc = if distribution_centers.is_empty() {
                    &warehouses[i % warehouses.len()]
                } else {
                    &distribution_centers[i % distribution_centers.len()]
                };
                let retail = locations.iter().find(|l| l.location_type == "RetailStore").unwrap();
                (dc.uri.clone(), retail.uri.clone(), "urn:epcglobal:cbv:bizstep:receiving")
            };
            
            events.push(EpcisEvent {
                uri: format!("http://example.com/event/logistics/{}", Uuid::new_v4()),
                event_type: EventType::ObjectEvent,
                event_time: event_time.to_rfc3339(),
                record_time: (event_time + chrono::Duration::minutes(10)).to_rfc3339(),
                event_id: format!("LOGIS-{:08}", i + 1),
                action: "OBSERVE".to_string(),
                biz_step: biz_step.to_string(),
                disposition: "urn:epcglobal:cbv:disp:in_transit".to_string(),
                epc_list: vec![product.epc.clone()],
                read_point: Some(format!("{}/dock{}", to_location, i % 5 + 1)),
                biz_location: Some(from_location),
                quantity: Some(1),
                business_transaction_list: vec![BusinessTransaction {
                    transaction_type: "urn:epcglobal:cbv:btt:po".to_string(),
                    transaction_id: format!("PO-{:08}", i + 1),
                }],
            });
        }
        
        Ok(events)
    }
    
    /// Generate retail events (pricing, displaying, selling)
    fn generate_retail_events(
        &self,
        products: &[Product],
        locations: &[Location],
        count: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        let retail_stores: Vec<&Location> = locations.iter().filter(|l| l.location_type == "RetailStore").collect();
        
        if retail_stores.is_empty() {
            return Ok(events);
        }
        
        for i in 0..count {
            let product = &products[i % products.len()];
            let store = &retail_stores[i % retail_stores.len()];
            let event_time = Utc::now() - chrono::Duration::days((count / 4 - i) as i64);
            
            let (biz_step, disposition, action) = match i % 4 {
                0 => (
                    "urn:epcglobal:cbv:bizstep:pricing",
                    "urn:epcglobal:cbv:disp:active",
                    "OBSERVE"
                ),
                1 => (
                    "urn:epcglobal:cbv:bizstep:displaying",
                    "urn:epcglobal:cbv:disp:active",
                    "OBSERVE"
                ),
                2 => (
                    "urn:epcglobal:cbv:bizstep:selling",
                    "urn:epcglobal:cbv:disp:sold",
                    "DELETE"
                ),
                3 => (
                    "urn:epcglobal:cbv:bizstep:customer_pickup",
                    "urn:epcglobal:cbv:disp:owned",
                    "DELETE"
                ),
                _ => (
                    "urn:epcglobal:cbv:bizstep:pricing",
                    "urn:epcglobal:cbv:disp:active",
                    "OBSERVE"
                ),
            };
            
            events.push(EpcisEvent {
                uri: format!("http://example.com/event/retail/{}", Uuid::new_v4()),
                event_type: EventType::ObjectEvent,
                event_time: event_time.to_rfc3339(),
                record_time: (event_time + chrono::Duration::minutes(2)).to_rfc3339(),
                event_id: format!("RETAIL-{:08}", i + 1),
                action: action.to_string(),
                biz_step: biz_step.to_string(),
                disposition: disposition.to_string(),
                epc_list: vec![product.epc.clone()],
                read_point: Some(format!("{}/shelf{}", store.uri, i % 20 + 1)),
                biz_location: Some(store.uri.clone()),
                quantity: Some(1),
                business_transaction_list: if action == "DELETE" {
                    vec![BusinessTransaction {
                        transaction_type: "urn:epcglobal:cbv:btt:inv".to_string(),
                        transaction_id: format!("INV-{:08}", i + 1),
                    }]
                } else {
                    vec![]
                },
            });
        }
        
        Ok(events)
    }
    
    /// Generate quality control events
    fn generate_quality_control_events(
        &self,
        products: &[Product],
        locations: &[Location],
        count: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        
        for i in 0..count {
            let product = &products[i % products.len()];
            let location = &locations[i % locations.len()];
            let event_time = Utc::now() - chrono::Duration::days((count / 6 - i) as i64);
            
            let (biz_step, disposition) = if i % 5 == 0 {
                // Failed inspection
                ("urn:epcglobal:cbv:bizstep:testing", "urn:epcglobal:cbv:disp:damaged")
            } else {
                // Passed inspection
                ("urn:epcglobal:cbv:bizstep:quality_control", "urn:epcglobal:cbv:disp:certified")
            };
            
            events.push(EpcisEvent {
                uri: format!("http://example.com/event/quality/{}", Uuid::new_v4()),
                event_type: EventType::ObjectEvent,
                event_time: event_time.to_rfc3339(),
                record_time: (event_time + chrono::Duration::minutes(15)).to_rfc3339(),
                event_id: format!("QUAL-{:08}", i + 1),
                action: "OBSERVE".to_string(),
                biz_step: biz_step.to_string(),
                disposition: disposition.to_string(),
                epc_list: vec![product.epc.clone()],
                read_point: Some(format!("{}/qc{}", location.uri, i % 3 + 1)),
                biz_location: Some(location.uri.clone()),
                quantity: Some(1),
                business_transaction_list: vec![],
            });
        }
        
        Ok(events)
    }
    
    /// Simulate a complete product journey through the supply chain
    pub fn simulate_product_journey(
        &self,
        product: &Product,
        locations: &[Location],
        journey_steps: usize,
    ) -> Result<Vec<EpcisEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        let mut current_time = Utc::now() - chrono::Duration::days(30);
        
        // Find relevant locations for the journey
        let factory = locations.iter().find(|l| l.location_type == "Factory").unwrap();
        let warehouse = locations.iter().find(|l| l.location_type == "Warehouse").unwrap();
        let distribution_center = locations.iter().find(|l| l.location_type == "DistributionCenter").unwrap_or(warehouse);
        let retail_store = locations.iter().find(|l| l.location_type == "RetailStore").unwrap();
        
        let journey_locations = vec![
            (factory.uri.clone(), "urn:epcglobal:cbv:bizstep:manufacturing"),
            (warehouse.uri.clone(), "urn:epcglobal:cbv:bizstep:receiving"),
            (distribution_center.uri.clone(), "urn:epcglobal:cbv:bizstep:storing"),
            (retail_store.uri.clone(), "urn:epcglobal:cbv:bizstep:displaying"),
        ];
        
        for (i, (location_uri, biz_step)) in journey_locations.iter().enumerate().take(journey_steps) {
            current_time += chrono::Duration::hours(i as i64 * 24);
            
            events.push(EpcisEvent {
                uri: format!("http://example.com/event/journey/{}", Uuid::new_v4()),
                event_type: EventType::ObjectEvent,
                event_time: current_time.to_rfc3339(),
                record_time: (current_time + chrono::Duration::minutes(5)).to_rfc3339(),
                event_id: format!("JOURNEY-{:08}-{:02}", i + 1, journey_steps),
                action: if i == 0 { "ADD" } else { "OBSERVE" }.to_string(),
                biz_step: biz_step.to_string(),
                disposition: "urn:epcglobal:cbv:disp:in_progress".to_string(),
                epc_list: vec![product.epc.clone()],
                read_point: Some(format!("{}/step{}", location_uri, i + 1)),
                biz_location: Some(location_uri.clone()),
                quantity: Some(1),
                business_transaction_list: vec![],
            });
        }
        
        Ok(events)
    }
}