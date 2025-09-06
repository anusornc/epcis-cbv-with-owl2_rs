use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Location in the supply chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub name: String,
    pub location_type: String,
    pub address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub capacity: Option<u32>,
    pub parent_location: Option<String>,
}

/// Product with EPC identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub uri: String,
    pub name: String,
    pub epc: String,
    pub product_type: String,
    pub category: String,
    pub manufacturer: String,
    pub manufacturing_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub weight_kg: Option<f64>,
    pub dimensions: Option<(f64, f64, f64)>, // length, width, height
}

/// Business entity (company, organization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessEntity {
    pub uri: String,
    pub name: String,
    pub entity_type: String,
    pub tax_id: Option<String>,
    pub contact_info: Option<ContactInfo>,
    pub locations: Vec<String>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

/// Generator for supply chain locations
pub struct LocationGenerator {
    location_types: Vec<String>,
    city_names: Vec<String>,
    country_names: Vec<String>,
}

impl LocationGenerator {
    pub fn new() -> Self {
        Self {
            location_types: vec![
                "Factory".to_string(),
                "Warehouse".to_string(),
                "DistributionCenter".to_string(),
                "RetailStore".to_string(),
                "ShippingPort".to_string(),
                "Airport".to_string(),
                "CrossDock".to_string(),
                "ProcessingFacility".to_string(),
            ],
            city_names: vec![
                "New York".to_string(),
                "Los Angeles".to_string(),
                "Chicago".to_string(),
                "Houston".to_string(),
                "Phoenix".to_string(),
                "Philadelphia".to_string(),
                "San Antonio".to_string(),
                "San Diego".to_string(),
                "Dallas".to_string(),
                "San Jose".to_string(),
            ],
            country_names: vec![
                "USA".to_string(),
                "Canada".to_string(),
                "Mexico".to_string(),
                "China".to_string(),
                "Germany".to_string(),
                "Japan".to_string(),
                "UK".to_string(),
                "France".to_string(),
            ],
        }
    }
    
    /// Generate a supply chain network with hierarchical locations
    pub fn generate_supply_chain_network(&self, count: usize) -> Result<Vec<Location>, Box<dyn std::error::Error + Send + Sync>> {
        let mut locations = Vec::new();
        
        // Generate regional distribution centers
        let regional_count = (count as f64 * 0.1).max(1.0) as usize;
        for i in 0..regional_count {
            locations.push(self.generate_location("DistributionCenter", i, None));
        }
        
        // Generate warehouses
        let warehouse_count = (count as f64 * 0.3).max(1.0) as usize;
        for i in 0..warehouse_count {
            let parent = if regional_count > 0 {
                Some(locations[i % regional_count].uri.clone())
            } else {
                None
            };
            locations.push(self.generate_location("Warehouse", i, parent));
        }
        
        // Generate factories
        let factory_count = (count as f64 * 0.2).max(1.0) as usize;
        for i in 0..factory_count {
            locations.push(self.generate_location("Factory", i, None));
        }
        
        // Generate retail stores
        let retail_count = count - regional_count - warehouse_count - factory_count;
        for i in 0..retail_count {
            let parent = if warehouse_count > 0 {
                Some(locations[regional_count + (i % warehouse_count)].uri.clone())
            } else {
                None
            };
            locations.push(self.generate_location("RetailStore", i, parent));
        }
        
        Ok(locations)
    }
    
    fn generate_location(&self, location_type: &str, index: usize, parent: Option<String>) -> Location {
        let city_index = index % self.city_names.len();
        let _country_index = (index / self.city_names.len()) % self.country_names.len();
        
        let name = format!("{} {} - {}", location_type, index + 1, self.city_names[city_index]);
        let uri = format!("http://example.com/location/{}", Uuid::new_v4());
        
        let coordinates = if index % 3 == 0 {
            Some((
                40.0 + (index as f64 * 0.1),
                -74.0 + (index as f64 * 0.1),
            ))
        } else {
            None
        };
        
        let capacity = match location_type {
            "Warehouse" => Some((10_000 + (index * 5_000)) as u32),
            "DistributionCenter" => Some((50_000 + (index * 10_000)) as u32),
            "RetailStore" => Some((1_000 + (index * 500)) as u32),
            "Factory" => Some((100_000 + (index * 20_000)) as u32),
            _ => None,
        };
        
        Location {
            uri,
            name,
            location_type: location_type.to_string(),
            address: Some(format!("{} {}, {}", index + 1, "Main St", self.city_names[city_index])),
            coordinates,
            capacity,
            parent_location: parent,
        }
    }
}

/// Generator for product catalog
pub struct ProductGenerator {
    product_categories: Vec<String>,
    product_types: Vec<String>,
    manufacturers: Vec<String>,
}

impl ProductGenerator {
    pub fn new() -> Self {
        Self {
            product_categories: vec![
                "Electronics".to_string(),
                "Clothing".to_string(),
                "Food".to_string(),
                "Beverages".to_string(),
                "Pharmaceuticals".to_string(),
                "Automotive".to_string(),
                "Furniture".to_string(),
                "Books".to_string(),
                "Toys".to_string(),
                "Sports".to_string(),
            ],
            product_types: vec![
                "Smartphone".to_string(),
                "Laptop".to_string(),
                "T-Shirt".to_string(),
                "Jeans".to_string(),
                "Coffee".to_string(),
                "Wine".to_string(),
                "Medicine".to_string(),
                "CarPart".to_string(),
                "Chair".to_string(),
                "Novel".to_string(),
            ],
            manufacturers: vec![
                "TechCorp".to_string(),
                "FashionInc".to_string(),
                "FoodCo".to_string(),
                "PharmaLtd".to_string(),
                "AutoGroup".to_string(),
                "FurnitureWorld".to_string(),
                "BookHouse".to_string(),
                "ToyFactory".to_string(),
                "SportsGear".to_string(),
            ],
        }
    }
    
    /// Generate product catalog with EPC codes
    pub fn generate_product_catalog(&self, count: usize) -> Result<Vec<Product>, Box<dyn std::error::Error + Send + Sync>> {
        let mut products = Vec::new();
        
        for i in 0..count {
            products.push(self.generate_product(i));
        }
        
        Ok(products)
    }
    
    fn generate_product(&self, index: usize) -> Product {
        let category_index = index % self.product_categories.len();
        let type_index = index % self.product_types.len();
        let manufacturer_index = index % self.manufacturers.len();
        
        let name = format!("{} {} {}", self.product_types[type_index], index + 1, self.manufacturers[manufacturer_index]);
        let uri = format!("http://example.com/product/{}", Uuid::new_v4());
        let epc = self.generate_epc_code(index);
        
        let manufacturing_date = if index % 10 != 0 {
            Some(Utc::now() - chrono::Duration::days((index * 30) as i64))
        } else {
            None
        };
        
        let expiration_date = if category_index == 3 || category_index == 4 { // Beverages or Pharmaceuticals
            Some(Utc::now() + chrono::Duration::days(365 + (index * 10) as i64))
        } else {
            None
        };
        
        Product {
            uri,
            name,
            epc,
            product_type: self.product_types[type_index].clone(),
            category: self.product_categories[category_index].clone(),
            manufacturer: self.manufacturers[manufacturer_index].clone(),
            manufacturing_date,
            expiration_date,
            weight_kg: Some(0.5 + (index as f64 * 0.1)),
            dimensions: Some((10.0 + (index as f64), 5.0 + (index as f64 * 0.5), 2.0 + (index as f64 * 0.2))),
        }
    }
    
    fn generate_epc_code(&self, index: usize) -> String {
        // Generate SGTIN (Serialized Global Trade Item Number) EPC
        let company_prefix = "0614141";
        let item_reference = format!("{:06}", index % 1000000);
        let serial_number = format!("{:08}", index % 100000000);
        format!("urn:epc:id:sgtin:{}.{}.{}", company_prefix, item_reference, serial_number)
    }
}

/// Generator for business entities
pub struct BusinessEntityGenerator {
    entity_types: Vec<String>,
}

impl BusinessEntityGenerator {
    pub fn new() -> Self {
        Self {
            entity_types: vec![
                "Manufacturer".to_string(),
                "Distributor".to_string(),
                "Retailer".to_string(),
                "LogisticsProvider".to_string(),
                "WarehouseOperator".to_string(),
            ],
        }
    }
    
    /// Generate business entities
    pub fn generate_business_entities(&self, count: usize) -> Result<Vec<BusinessEntity>, Box<dyn std::error::Error + Send + Sync>> {
        let mut entities = Vec::new();
        
        for i in 0..count {
            entities.push(self.generate_business_entity(i));
        }
        
        Ok(entities)
    }
    
    fn generate_business_entity(&self, index: usize) -> BusinessEntity {
        let entity_type = &self.entity_types[index % self.entity_types.len()];
        let name = format!("{} {}", entity_type, index + 1);
        let uri = format!("http://example.com/entity/{}", Uuid::new_v4());
        
        let contact_info = Some(ContactInfo {
            email: Some(format!("contact@{}{}.com", entity_type.to_lowercase(), index + 1)),
            phone: Some(format!("+1-555-{:03}-{:04}", (index % 1000) as u32, (index % 10000) as u32)),
            address: Some(format!("{} Business Ave, Suite {}", index + 1, index + 100)),
        });
        
        BusinessEntity {
            uri,
            name,
            entity_type: entity_type.clone(),
            tax_id: Some(format!("TAX-{:09}", index + 1)),
            contact_info,
            locations: Vec::new(), // Will be populated by location generator
        }
    }
}