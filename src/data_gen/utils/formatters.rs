use crate::data_gen::events::EpcisEvent;
use crate::data_gen::entities::{Location, Product, BusinessEntity};

/// Trait for formatting data into different RDF formats
pub trait DataFormatter {
    fn format_triples(&self, triples: &[oxrdf::Triple]) -> String;
    fn format_events(&self, events: &[EpcisEvent]) -> String;
    fn format_entities(&self, locations: &[Location], products: &[Product], entities: &[BusinessEntity]) -> String;
}

/// Turtle format formatter
pub struct TurtleFormatter;

impl TurtleFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter for TurtleFormatter {
    fn format_triples(&self, triples: &[oxrdf::Triple]) -> String {
        let mut output = String::new();
        
        // Add Turtle header
        output.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        output.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        output.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
        output.push_str("@prefix epcis: <urn:epcglobal:epcis:> .\n");
        output.push_str("@prefix cbv: <urn:epcglobal:cbv:> .\n");
        output.push_str("@prefix ex: <http://example.com/> .\n\n");
        
        // Format triples
        for triple in triples {
            output.push_str(&format!(
                "{} {} {} .\n",
                triple.subject,
                triple.predicate,
                self.format_object(&triple.object)
            ));
        }
        
        output
    }
    
    fn format_events(&self, events: &[EpcisEvent]) -> String {
        let mut output = String::new();
        
        output.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        output.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        output.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
        output.push_str("@prefix epcis: <urn:epcglobal:epcis:> .\n");
        output.push_str("@prefix cbv: <urn:epcglobal:cbv:> .\n");
        output.push_str("@prefix ex: <http://example.com/> .\n\n");
        
        for event in events {
            output.push_str(&format_event_turtle(event));
        }
        
        output
    }
    
    fn format_entities(&self, locations: &[Location], products: &[Product], entities: &[BusinessEntity]) -> String {
        let mut output = String::new();
        
        output.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        output.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        output.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
        output.push_str("@prefix epcis: <urn:epcglobal:epcis:> .\n");
        output.push_str("@prefix cbv: <urn:epcglobal:cbv:> .\n");
        output.push_str("@prefix ex: <http://example.com/> .\n\n");
        
        // Format locations
        for location in locations {
            output.push_str(&format_location_turtle(location));
        }
        
        // Format products
        for product in products {
            output.push_str(&format_product_turtle(product));
        }
        
        // Format business entities
        for entity in entities {
            output.push_str(&format_business_entity_turtle(entity));
        }
        
        output
    }
}

impl TurtleFormatter {
    fn format_object(&self, object: &oxrdf::Term) -> String {
        match object {
            oxrdf::Term::NamedNode(node) => format!("<{}>", node.as_ref()),
            oxrdf::Term::BlankNode(node) => format!("_:{}", node.as_str()),
            oxrdf::Term::Literal(literal) => {
                if literal.datatype() == oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#string").unwrap() {
                    format!("\"{}\"", literal.value())
                } else if literal.datatype() == oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#dateTime").unwrap() {
                    format!("\"{}\"^^xsd:dateTime", literal.value())
                } else if literal.datatype() == oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap() {
                    format!("\"{}\"^^xsd:integer", literal.value())
                } else {
                    format!("\"{}\"", literal.value())
                }
            }
        }
    }
}

/// N-Triples format formatter
pub struct NTriplesFormatter;

impl NTriplesFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter for NTriplesFormatter {
    fn format_triples(&self, triples: &[oxrdf::Triple]) -> String {
        let mut output = String::new();
        
        for triple in triples {
            output.push_str(&format!(
                "{} {} {} .\n",
                triple.subject,
                triple.predicate,
                format_ntriples_object(&triple.object)
            ));
        }
        
        output
    }
    
    fn format_events(&self, events: &[EpcisEvent]) -> String {
        let mut output = String::new();
        
        for event in events {
            let event_triples = convert_event_to_triples(event);
            for triple in event_triples {
                output.push_str(&format!(
                    "{} {} {} .\n",
                    triple.subject,
                    triple.predicate,
                    format_ntriples_object(&triple.object)
                ));
            }
        }
        
        output
    }
    
    fn format_entities(&self, locations: &[Location], products: &[Product], entities: &[BusinessEntity]) -> String {
        let mut output = String::new();
        
        for location in locations {
            let location_triples = convert_location_to_triples(location);
            for triple in location_triples {
                output.push_str(&format!(
                    "{} {} {} .\n",
                    triple.subject,
                    triple.predicate,
                    format_ntriples_object(&triple.object)
                ));
            }
        }
        
        for product in products {
            let product_triples = convert_product_to_triples(product);
            for triple in product_triples {
                output.push_str(&format!(
                    "{} {} {} .\n",
                    triple.subject,
                    triple.predicate,
                    format_ntriples_object(&triple.object)
                ));
            }
        }
        
        for entity in entities {
            let entity_triples = convert_business_entity_to_triples(entity);
            for triple in entity_triples {
                output.push_str(&format!(
                    "{} {} {} .\n",
                    triple.subject,
                    triple.predicate,
                    format_ntriples_object(&triple.object)
                ));
            }
        }
        
        output
    }
}

/// JSON-LD format formatter  
pub struct JsonLdFormatter;

impl JsonLdFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter for JsonLdFormatter {
    fn format_triples(&self, triples: &[oxrdf::Triple]) -> String {
        let mut graph = Vec::new();
        
        for triple in triples {
            let subject_str = match &triple.subject {
                oxrdf::Subject::NamedNode(node) => node.as_str(),
                oxrdf::Subject::BlankNode(node) => node.as_str(),
            };
            
            graph.push(serde_json::json!({
                "subject": subject_str,
                "predicate": triple.predicate.as_str(),
                "object": format_jsonld_object(&triple.object)
            }));
        }
        
        let jsonld = serde_json::json!({
            "@context": {
                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "xsd": "http://www.w3.org/2001/XMLSchema#",
                "epcis": "urn:epcglobal:epcis:",
                "cbv": "urn:epcglobal:cbv:",
                "ex": "http://example.com/"
            },
            "@graph": graph
        });
        
        serde_json::to_string_pretty(&jsonld).unwrap_or_default()
    }
    
    fn format_events(&self, events: &[EpcisEvent]) -> String {
        let mut graph = Vec::new();
        
        for event in events {
            let event_json = convert_event_to_jsonld(event);
            graph.push(event_json);
        }
        
        let jsonld = serde_json::json!({
            "@context": {
                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "xsd": "http://www.w3.org/2001/XMLSchema#",
                "epcis": "urn:epcglobal:epcis:",
                "cbv": "urn:epcglobal:cbv:",
                "ex": "http://example.com/"
            },
            "@graph": graph
        });
        
        serde_json::to_string_pretty(&jsonld).unwrap_or_default()
    }
    
    fn format_entities(&self, locations: &[Location], products: &[Product], entities: &[BusinessEntity]) -> String {
        let mut graph = Vec::new();
        
        for location in locations {
            graph.push(convert_location_to_jsonld(location));
        }
        
        for product in products {
            graph.push(convert_product_to_jsonld(product));
        }
        
        for entity in entities {
            graph.push(convert_business_entity_to_jsonld(entity));
        }
        
        let jsonld = serde_json::json!({
            "@context": {
                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "xsd": "http://www.w3.org/2001/XMLSchema#",
                "epcis": "urn:epcglobal:epcis:",
                "cbv": "urn:epcglobal:cbv:",
                "ex": "http://example.com/"
            },
            "@graph": graph
        });
        
        serde_json::to_string_pretty(&jsonld).unwrap_or_default()
    }
}

// Helper functions for formatting

fn format_event_turtle(event: &EpcisEvent) -> String {
    format!(
        r#"<{}> rdf:type epcis:ObjectEvent ;
    epcis:eventID "{}" ;
    epcis:eventTime "{}"^^xsd:dateTime ;
    epcis:recordTime "{}"^^xsd:dateTime ;
    epcis:action "{}" ;
    epcis:bizStep "{}" ;
    epcis:disposition "{}" ;
    epcis:epcList "{}" ;
{}
{}
{} .
"#,
        event.uri,
        event.event_id,
        event.event_time,
        event.record_time,
        event.action,
        event.biz_step,
        event.disposition,
        event.epc_list.join(", "),
        if let Some(ref read_point) = event.read_point {
            format!("    epcis:readPoint <{}> ;", read_point)
        } else {
            String::new()
        },
        if let Some(ref biz_location) = event.biz_location {
            format!("    epcis:bizLocation <{}> ;", biz_location)
        } else {
            String::new()
        },
        if let Some(quantity) = event.quantity {
            format!("    epcis:quantity {}^^xsd:integer", quantity)
        } else {
            String::new()
        }
    )
}

fn format_location_turtle(location: &Location) -> String {
    format!(
        r#"<{}> rdf:type ex:Location ;
    rdfs:label "{}" ;
    ex:name "{}" ;
    ex:locationType "{}" ;
{}{}
{} .
"#,
        location.uri,
        location.name,
        location.name,
        location.location_type,
        if let Some(ref address) = location.address {
            format!("    ex:address \"{}\" ;\n", address)
        } else {
            String::new()
        },
        if let Some((lat, lon)) = location.coordinates {
            format!("    ex:coordinates \"{}, {}\" ;", lat, lon)
        } else {
            String::new()
        },
        if let Some(capacity) = location.capacity {
            format!("    ex:capacity {}^^xsd:integer", capacity)
        } else {
            String::new()
        }
    )
}

fn format_product_turtle(product: &Product) -> String {
    format!(
        r#"<{}> rdf:type ex:Product ;
    rdfs:label "{}" ;
    ex:name "{}" ;
    ex:epc "{}" ;
    ex:productType "{}" ;
    ex:category "{}" ;
    ex:manufacturer "{}" ;
{}{}{} .
"#,
        product.uri,
        product.name,
        product.name,
        product.epc,
        product.product_type,
        product.category,
        product.manufacturer,
        if let Some(ref mfg_date) = product.manufacturing_date {
            format!("    ex:manufacturingDate \"{}\"^^xsd:dateTime ;\n", mfg_date.to_rfc3339())
        } else {
            String::new()
        },
        if let Some(ref exp_date) = product.expiration_date {
            format!("    ex:expirationDate \"{}\"^^xsd:dateTime ;\n", exp_date.to_rfc3339())
        } else {
            String::new()
        },
        if let Some(weight) = product.weight_kg {
            format!("    ex:weight {}^^xsd:decimal", weight)
        } else {
            String::new()
        }
    )
}

fn format_business_entity_turtle(entity: &BusinessEntity) -> String {
    format!(
        r#"<{}> rdf:type ex:BusinessEntity ;
    rdfs:label "{}" ;
    ex:name "{}" ;
    ex:entityType "{}" ;
{}{} .
"#,
        entity.uri,
        entity.name,
        entity.name,
        entity.entity_type,
        if let Some(ref tax_id) = entity.tax_id {
            format!("    ex:taxId \"{}\" ;\n", tax_id)
        } else {
            String::new()
        },
        if let Some(ref contact) = entity.contact_info {
            format_contact_info_turtle(contact)
        } else {
            String::new()
        }
    )
}

fn format_contact_info_turtle(contact: &crate::data_gen::entities::ContactInfo) -> String {
    let mut parts = Vec::new();
    
    if let Some(ref email) = contact.email {
        parts.push(format!("    ex:email \"{}\"", email));
    }
    if let Some(ref phone) = contact.phone {
        parts.push(format!("    ex:phone \"{}\"", phone));
    }
    if let Some(ref address) = contact.address {
        parts.push(format!("    ex:address \"{}\"", address));
    }
    
    parts.join(" ;\n")
}

fn format_ntriples_object(object: &oxrdf::Term) -> String {
    match object {
        oxrdf::Term::NamedNode(node) => format!("<{}>", node.as_str()),
        oxrdf::Term::BlankNode(node) => format!("_:{}", node.as_str()),
        oxrdf::Term::Literal(literal) => {
            if literal.datatype() == oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#string").unwrap() {
                format!("\"{}\"", literal.value())
            } else {
                format!("\"{}\"^^<{}>", literal.value(), literal.datatype().as_str())
            }
        }
    }
}

fn format_jsonld_object(object: &oxrdf::Term) -> serde_json::Value {
    match object {
        oxrdf::Term::NamedNode(node) => serde_json::json!({"@id": node.as_str()}),
        oxrdf::Term::BlankNode(node) => serde_json::json!({"@id": format!("_:{}", node.as_str())}),
        oxrdf::Term::Literal(literal) => {
            if literal.datatype() == oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#string").unwrap() {
                serde_json::json!({"@value": literal.value()})
            } else {
                serde_json::json!({
                    "@value": literal.value(),
                    "@type": literal.datatype().as_str()
                })
            }
        }
    }
}

// Conversion functions for different formatters
fn convert_event_to_triples(event: &EpcisEvent) -> Vec<oxrdf::Triple> {
    vec![
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&event.uri).unwrap(),
            oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:ObjectEvent").unwrap(),
        ),
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&event.uri).unwrap(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:eventTime").unwrap(),
            oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(&event.event_time)),
        ),
    ]
}

fn convert_location_to_triples(location: &Location) -> Vec<oxrdf::Triple> {
    vec![
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&location.uri).unwrap(),
            oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            oxrdf::NamedNode::new("http://example.com/Location").unwrap(),
        ),
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&location.uri).unwrap(),
            oxrdf::NamedNode::new("http://example.com/name").unwrap(),
            oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(&location.name)),
        ),
    ]
}

fn convert_product_to_triples(product: &Product) -> Vec<oxrdf::Triple> {
    vec![
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&product.uri).unwrap(),
            oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            oxrdf::NamedNode::new("http://example.com/Product").unwrap(),
        ),
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&product.uri).unwrap(),
            oxrdf::NamedNode::new("http://example.com/epc").unwrap(),
            oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(&product.epc)),
        ),
    ]
}

fn convert_business_entity_to_triples(entity: &BusinessEntity) -> Vec<oxrdf::Triple> {
    vec![
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&entity.uri).unwrap(),
            oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            oxrdf::NamedNode::new("http://example.com/BusinessEntity").unwrap(),
        ),
        oxrdf::Triple::new(
            oxrdf::NamedNode::new(&entity.uri).unwrap(),
            oxrdf::NamedNode::new("http://example.com/name").unwrap(),
            oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(&entity.name)),
        ),
    ]
}

fn convert_event_to_jsonld(event: &EpcisEvent) -> serde_json::Value {
    serde_json::json!({
        "@id": event.uri,
        "@type": "epcis:ObjectEvent",
        "epcis:eventID": event.event_id,
        "epcis:eventTime": {
            "@value": event.event_time,
            "@type": "xsd:dateTime"
        },
        "epcis:action": event.action,
        "epcis:bizStep": event.biz_step,
        "epcis:disposition": event.disposition
    })
}

fn convert_location_to_jsonld(location: &Location) -> serde_json::Value {
    serde_json::json!({
        "@id": location.uri,
        "@type": "ex:Location",
        "rdfs:label": location.name,
        "ex:name": location.name,
        "ex:locationType": location.location_type
    })
}

fn convert_product_to_jsonld(product: &Product) -> serde_json::Value {
    serde_json::json!({
        "@id": product.uri,
        "@type": "ex:Product",
        "rdfs:label": product.name,
        "ex:name": product.name,
        "ex:epc": product.epc,
        "ex:productType": product.product_type,
        "ex:category": product.category,
        "ex:manufacturer": product.manufacturer
    })
}

fn convert_business_entity_to_jsonld(entity: &BusinessEntity) -> serde_json::Value {
    serde_json::json!({
        "@id": entity.uri,
        "@type": "ex:BusinessEntity",
        "rdfs:label": entity.name,
        "ex:name": entity.name,
        "ex:entityType": entity.entity_type
    })
}