pub struct AssetSymbol(String); // BTC/USDT

pub enum AssetSymbolError {
    InvalidSymbol,
    InvalidSymbolFormReqPath,
    MarketNotSupported(String),
}

impl AssetSymbol {
    // from internal/request body format: "BTC/USDT"
    pub fn new(symbol: &str) -> Result<Self, AssetSymbolError> {
        // validate contains exactly one "/"
        Self::validate(symbol, "/")?;
        // normalize: trim, uppercase
        Ok(AssetSymbol(symbol.trim().to_uppercase()))
    }

    // from URL path: "BTC-USDT"
    pub fn from_path(symbol: &str) -> Result<Self, AssetSymbolError> {
        // validate contains exactly one "-"
        Self::validate(symbol, "-")?;

        // convert "-" to "/" then store
        let symbol = symbol.replace("-", "/");
        Ok(AssetSymbol(symbol.trim().to_ascii_uppercase()))
    }

    // returns "BTC/USDT" — for DB lookups
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn base_asset(&self) -> &str {
        &self.0.split("/").collect::<Vec<&str>>()[0]
    }

    pub fn quote_asset(&self) -> &str {
        &self.0.split("/").collect::<Vec<&str>>()[1]
    }

    fn validate(symbol: &str, pat: &str) -> Result<(), AssetSymbolError> {
        if symbol.contains(pat) {
            if symbol.split(pat).collect::<Vec<&str>>().len() != 2 {
                return Err(AssetSymbolError::InvalidSymbol);
            };
        } else {
            return Err(AssetSymbolError::InvalidSymbol);
        }

        Ok(())
    }
}
