use cosmwasm_std::Decimal;
use oraiswap::asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub owner: String,
    pub mint_contract: String,
    pub base_denom: String,
    pub oraix_oracle: String,
    pub anchor_oracle: String,
    pub band_oracle: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<String>,
        mint_contract: Option<String>,
        base_denom: Option<String>,
        oraix_oracle: Option<String>,
        anchor_oracle: Option<String>,
        band_oracle: Option<String>,
    },
    RegisterCollateralAsset {
        asset: AssetInfo,
        price_source: SourceType,
        multiplier: Decimal,
    },
    RevokeCollateralAsset {
        asset: AssetInfo,
    },
    UpdateCollateralPriceSource {
        asset: AssetInfo,
        price_source: SourceType,
    },
    UpdateCollateralMultiplier {
        asset: AssetInfo,
        multiplier: Decimal,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    CollateralPrice {
        asset: String,
        block_height: Option<u64>,
    },
    CollateralAssetInfo {
        asset: String,
    },
    CollateralAssetInfos {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub mint_contract: String,
    pub base_denom: String,
    pub oraix_oracle: String,
    pub anchor_oracle: String,
    pub band_oracle: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralPriceResponse {
    pub asset: String,
    pub rate: Decimal,
    pub last_updated: u64,
    pub multiplier: Decimal,
    pub is_revoked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralInfoResponse {
    pub asset: String,
    pub multiplier: Decimal,
    pub source_type: String,
    pub is_revoked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralInfosResponse {
    pub collaterals: Vec<CollateralInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub lunax_token_addr: String,
    pub lunax_staking_contract: String,
    pub multiplier: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    MirrorOracle {},
    AnchorOracle {},
    BandOracle {},
    FixedPrice {
        price: Decimal,
    },
    Terraswap {
        oraiswap_pair_addr: String,
        intermediate_denom: Option<String>,
    },
    AnchorMarket {
        anchor_market_addr: String,
    },
    Native {
        native_denom: String,
    },
    Lunax {
        staking_contract_addr: String,
    },
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SourceType::MirrorOracle { .. } => write!(f, "oraix_oracle"),
            SourceType::AnchorOracle { .. } => write!(f, "anchor_oracle"),
            SourceType::BandOracle { .. } => write!(f, "band_oracle"),
            SourceType::FixedPrice { .. } => write!(f, "fixed_price"),
            SourceType::Terraswap { .. } => write!(f, "oraiswap"),
            SourceType::AnchorMarket { .. } => write!(f, "anchor_market"),
            SourceType::Native { .. } => write!(f, "native"),
            SourceType::Lunax { .. } => write!(f, "lunax"),
        }
    }
}