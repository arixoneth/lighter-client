use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CreateOrder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_book_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ask: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl CreateOrder {
    pub fn from_json_str(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CancelOrder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_book_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl CancelOrder {
    pub fn from_json_str(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Withdraw {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collateral_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl Withdraw {
    pub fn from_json_str(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CreateGroupedOrders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_book_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl CreateGroupedOrders {
    pub fn from_json_str(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
