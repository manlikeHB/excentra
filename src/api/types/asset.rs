use crate::error::AppError;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AddAssetRequest {
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
}

impl AddAssetRequest {
    pub fn normalize(&self) -> Result<Self, AppError> {
        // normalize asset name
        let lower = self.name.to_lowercase();
        let mut chars = lower.chars();

        let name = match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => {
                return Err(AppError::BadRequest(
                    "Asset name can not be empty, e.g 'BTC".to_string(),
                ));
            }
        };

        // normalize asset symbol
        let symbol = self.symbol.to_uppercase();

        Ok(AddAssetRequest {
            symbol,
            name,
            decimals: self.decimals,
        })
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.symbol.is_empty() {
            return Err(AppError::BadRequest(
                "Asset symbol can not be empty, e.g 'BTC'".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(AppError::BadRequest(
                "Asset name can not be empty, e.g 'BTC'".to_string(),
            ));
        }

        if self.decimals <= 0 {
            return Err(AppError::BadRequest(
                "Asset decimals can not be less than zero".to_string(),
            ));
        }

        Ok(())
    }
}
