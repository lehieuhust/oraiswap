use crate::orderbook::Order;
use crate::state::{
    increase_last_order_id, read_last_order_id, read_order, read_orders,
    read_orders_with_bidder_indexer, remove_order, store_order,
};
use cosmwasm_std::{
    Addr, CosmosMsg, Decimal, Deps, DepsMut, MessageInfo, Order as OrderBy, Response, StdError,
    StdResult, Uint128,
};

use oraiswap::asset::{pair_key, Asset, AssetInfo};
use oraiswap::limit_order::{LastOrderIdResponse, OrderDirection, OrderResponse, OrdersResponse};

pub fn submit_order(
    deps: DepsMut,
    sender: Addr,
    order_direction: OrderDirection,
    offer_asset: Asset,
    ask_asset: Asset,
) -> StdResult<Response> {
    let order_id = increase_last_order_id(deps.storage)?;

    let offer_asset_raw = offer_asset.to_raw(deps.api)?;
    let ask_asset_raw = ask_asset.to_raw(deps.api)?;
    let pair_key = pair_key(&[offer_asset_raw.info, ask_asset_raw.info]);
    store_order(
        deps.storage,
        &pair_key,
        &Order {
            order_id,
            direction: order_direction,
            bidder_addr: deps.api.addr_canonicalize(sender.as_str())?,
            offer_amount: offer_asset_raw.amount,
            ask_amount: ask_asset_raw.amount,
            filled_offer_amount: Uint128::zero(),
            filled_ask_amount: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "submit_order"),
        ("order_id", &order_id.to_string()),
        ("bidder_addr", sender.as_str()),
        ("offer_asset", &offer_asset.to_string()),
        ("ask_asset", &ask_asset.to_string()),
    ]))
}

pub fn cancel_order(
    deps: DepsMut,
    info: MessageInfo,
    offer_info: AssetInfo,
    ask_info: AssetInfo,
    order_id: u64,
) -> StdResult<Response> {
    let pair_key = pair_key(&[offer_info.to_raw(deps.api)?, ask_info.to_raw(deps.api)?]);
    let order: Order = read_order(deps.storage, &pair_key, order_id)?;
    if order.bidder_addr != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Compute refund asset
    let left_offer_amount = order.offer_amount.checked_sub(order.filled_offer_amount)?;
    let bidder_refund = Asset {
        info: offer_info,
        amount: left_offer_amount,
    };

    // Build refund msg
    let messages = if left_offer_amount > Uint128::zero() {
        vec![bidder_refund
            .clone()
            .into_msg(None, &deps.querier, info.sender)?]
    } else {
        vec![]
    };

    remove_order(deps.storage, &pair_key, &order);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "cancel_order"),
        ("order_id", &order_id.to_string()),
        ("bidder_refund", &bidder_refund.to_string()),
    ]))
}

pub fn execute_order(
    deps: DepsMut,
    offer_info: AssetInfo,
    sender: Addr,
    ask_asset: Asset,
    order_id: u64,
) -> StdResult<Response> {
    let pair_key = pair_key(&[
        offer_info.to_raw(deps.api)?,
        ask_asset.info.to_raw(deps.api)?,
    ]);
    let mut order: Order = read_order(deps.storage, &pair_key, order_id)?;

    // Compute left offer & ask amount
    let left_offer_amount = order.offer_amount.checked_sub(order.filled_offer_amount)?;
    let left_ask_amount = order.ask_amount.checked_sub(order.filled_ask_amount)?;
    if left_ask_amount < ask_asset.amount || left_offer_amount.is_zero() {
        return Err(StdError::generic_err("insufficient order amount left"));
    }

    // Cap the send amount to left_offer_amount
    let executor_receive = Asset {
        info: offer_info,
        amount: if left_ask_amount == ask_asset.amount {
            left_offer_amount
        } else {
            std::cmp::min(
                left_offer_amount,
                ask_asset.amount * Decimal::from_ratio(order.offer_amount, order.ask_amount),
            )
        },
    };

    let bidder_addr = deps.api.addr_humanize(&order.bidder_addr)?;
    let bidder_receive = ask_asset;

    // When left amount is zero, close order
    if left_ask_amount == bidder_receive.amount {
        remove_order(deps.storage, &pair_key, &order);
    } else {
        order.filled_ask_amount += bidder_receive.amount;
        order.filled_offer_amount += executor_receive.amount;
        store_order(deps.storage, &pair_key, &order)?;
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    if !executor_receive.amount.is_zero() {
        // dont use oracle for limit order
        messages.push(executor_receive.clone().into_msg(
            None,
            &deps.querier,
            deps.api.addr_validate(sender.as_str())?,
        )?);
    }

    if !bidder_receive.amount.is_zero() {
        messages.push(
            bidder_receive
                .clone()
                .into_msg(None, &deps.querier, bidder_addr)?,
        );
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "execute_order"),
        ("order_id", &order_id.to_string()),
        ("executor_receive", &executor_receive.to_string()),
        ("bidder_receive", &bidder_receive.to_string()),
    ]))
}

pub fn query_order(
    deps: Deps,
    offer_info: AssetInfo,
    ask_info: AssetInfo,
    order_id: u64,
) -> StdResult<OrderResponse> {
    let pair_key = pair_key(&[offer_info.to_raw(deps.api)?, ask_info.to_raw(deps.api)?]);
    let order: Order = read_order(deps.storage, &pair_key, order_id)?;
    let resp = OrderResponse {
        order_id: order.order_id,
        bidder_addr: deps.api.addr_humanize(&order.bidder_addr)?.to_string(),
        offer_asset: Asset {
            amount: order.offer_amount,
            info: offer_info,
        },
        ask_asset: Asset {
            amount: order.ask_amount,
            info: ask_info,
        },
        filled_offer_amount: order.filled_offer_amount,
        filled_ask_amount: order.filled_ask_amount,
    };

    Ok(resp)
}

pub fn query_orders(
    deps: Deps,
    offer_info: AssetInfo,
    ask_info: AssetInfo,
    bidder_addr: Option<String>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<OrdersResponse> {
    let pair_key = pair_key(&[offer_info.to_raw(deps.api)?, ask_info.to_raw(deps.api)?]);
    let orders: Vec<Order> = if let Some(bidder_addr) = bidder_addr {
        let bidder_addr_raw = deps.api.addr_canonicalize(&bidder_addr)?;
        read_orders_with_bidder_indexer(
            deps.storage,
            &bidder_addr_raw,
            &pair_key,
            start_after,
            limit,
            order_by,
        )?
    } else {
        read_orders(deps.storage, &pair_key, start_after, limit, order_by)?
    };

    let resp = OrdersResponse {
        orders: orders
            .iter()
            .map(|order| {
                Ok(OrderResponse {
                    order_id: order.order_id,
                    bidder_addr: deps.api.addr_humanize(&order.bidder_addr)?.to_string(),
                    offer_asset: Asset {
                        amount: order.offer_amount,
                        info: offer_info.clone(),
                    },
                    ask_asset: Asset {
                        amount: order.ask_amount,
                        info: ask_info.clone(),
                    },
                    filled_offer_amount: order.filled_offer_amount,
                    filled_ask_amount: order.filled_ask_amount,
                })
            })
            .collect::<StdResult<Vec<OrderResponse>>>()?,
    };

    Ok(resp)
}

pub fn query_last_order_id(deps: Deps) -> StdResult<LastOrderIdResponse> {
    let last_order_id = read_last_order_id(deps.storage)?;
    let resp = LastOrderIdResponse { last_order_id };

    Ok(resp)
}
