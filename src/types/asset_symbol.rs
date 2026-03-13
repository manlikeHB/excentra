#[derive(Debug, PartialEq, Eq)]
pub struct AssetSymbol(String); // BTC/USDT

#[derive(Debug, PartialEq, Eq)]
pub enum AssetSymbolError {
    InvalidSymbol,
    InvalidSymbolFormReqPath,
    MarketNotSupported(String),
}

pub enum SymbolPattern {
    Hyphen,
    Slash,
}

impl AssetSymbol {
    // from internal/request body format: "BTC/USDT"
    pub fn new(symbol: &str) -> Result<Self, AssetSymbolError> {
        // validate contains exactly one "/"
        Self::validate(symbol, SymbolPattern::Slash)?;

        // normalize: trim, uppercase
        let res: Vec<&str> = symbol.split("/").map(|part| part.trim()).collect();
        Ok(AssetSymbol(
            format!("{}/{}", res[0], res[1]).to_ascii_uppercase(),
        ))
    }

    // from URL path: "BTC-USDT"
    pub fn from_path(symbol: &str) -> Result<Self, AssetSymbolError> {
        // validate contains exactly one "-"
        Self::validate(symbol, SymbolPattern::Hyphen)?;

        // convert "-" to "/" then store
        let res: Vec<&str> = symbol.split("-").map(|part| part.trim()).collect();
        Ok(AssetSymbol(
            format!("{}/{}", res[0], res[1]).to_ascii_uppercase(),
        ))
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

    fn validate(symbol: &str, pat: SymbolPattern) -> Result<(), AssetSymbolError> {
        match pat {
            SymbolPattern::Hyphen => {
                if symbol.contains("-") {
                    if symbol.split("-").collect::<Vec<&str>>().len() != 2 {
                        return Err(AssetSymbolError::InvalidSymbolFormReqPath);
                    };
                } else {
                    return Err(AssetSymbolError::InvalidSymbolFormReqPath);
                }
            }
            SymbolPattern::Slash => {
                if symbol.contains("/") {
                    if symbol.split("/").collect::<Vec<&str>>().len() != 2 {
                        return Err(AssetSymbolError::InvalidSymbol);
                    };
                } else {
                    return Err(AssetSymbolError::InvalidSymbol);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // AssetSymbol::new — internal format (BTC/USDT)
    // ============================================================

    #[test]
    fn test_new_valid_symbol() {
        assert!(AssetSymbol::new("BTC/USDT").is_ok());
    }

    #[test]
    fn test_new_valid_symbol_lowercased_gets_uppercased() {
        let res = AssetSymbol::new("btc/usdt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().as_str(), "BTC/USDT");
    }

    #[test]
    fn test_new_valid_symbol_with_whitespace_gets_trimmed() {
        let res = AssetSymbol::new("  btc / usdt  ");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().as_str(), "BTC/USDT");
    }

    #[test]
    fn test_new_missing_separator_fails() {
        assert!(AssetSymbol::new("btcusdt").is_err());
    }

    #[test]
    fn test_new_wrong_separator_fails() {
        assert!(AssetSymbol::new("BTC-USDT").is_err());
    }

    #[test]
    fn test_new_multiple_slashes_fails() {
        assert!(AssetSymbol::new("BTC/USDT/ETH").is_err());
    }

    #[test]
    fn test_new_empty_string_fails() {
        assert!(AssetSymbol::new(" ").is_err());
    }

    // ============================================================
    // AssetSymbol::from_path — URL format (BTC-USDT)
    // ============================================================

    #[test]
    fn test_from_path_valid_symbol() {
        let res = AssetSymbol::from_path("btc-usdt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().as_str(), "BTC/USDT")
    }

    #[test]
    fn test_from_path_converts_dash_to_slash_internally() {
        let res = AssetSymbol::from_path("btc-usdt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().as_str(), "BTC/USDT")
    }

    #[test]
    fn test_from_path_lowercased_gets_uppercased() {
        let res = AssetSymbol::from_path("btc-usdt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().as_str(), "BTC/USDT")
    }

    #[test]
    fn test_from_path_wrong_separator_fails() {
        let res = AssetSymbol::from_path("btc/usdt");
        assert!(res.is_err());
    }

    #[test]
    fn test_from_path_multiple_dashes_fails() {
        assert!(AssetSymbol::from_path("BTC-USDT-ETH").is_err());
    }

    #[test]
    fn test_from_path_empty_string_fails() {
        assert!(AssetSymbol::from_path(" ").is_err());
    }

    // ============================================================
    // as_str — always returns internal format
    // ============================================================

    #[test]
    fn test_as_str_returns_slash_format() {
        assert_eq!(AssetSymbol::new("BTC/USDT").unwrap().as_str(), "BTC/USDT");
    }

    #[test]
    fn test_from_path_as_str_returns_slash_not_dash() {
        assert_eq!(
            AssetSymbol::from_path("btc-USDT").unwrap().as_str(),
            "BTC/USDT"
        );
    }

    // ============================================================
    // base_asset / quote_asset
    // ============================================================

    #[test]
    fn test_base_asset() {
        let res = AssetSymbol::new("BTC/USDT");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().base_asset(), "BTC");
    }

    #[test]
    fn test_quote_asset() {
        let res = AssetSymbol::new("BTC/USDT");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().quote_asset(), "USDT");
    }

    // ============================================================
    // Error variants
    // ============================================================

    #[test]
    fn test_new_returns_invalid_symbol_error() {
        assert_eq!(
            AssetSymbol::new("btcusdt").unwrap_err(),
            AssetSymbolError::InvalidSymbol
        );
    }

    #[test]
    fn test_from_path_returns_invalid_path_error() {
        assert_eq!(
            AssetSymbol::from_path("btcusdt").unwrap_err(),
            AssetSymbolError::InvalidSymbolFormReqPath
        );
    }
}
