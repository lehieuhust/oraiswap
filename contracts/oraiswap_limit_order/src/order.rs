use std::convert::TryFrom;
use std::str::FromStr;

use crate::orderbook::{Executor, Order};
use crate::state::{
    increase_last_order_id, read_config, read_last_order_id, read_order, read_orderbook,
    read_orderbooks, read_orders, read_orders_with_indexer, read_reward, remove_order,
    remove_orderbook, store_order, store_reward, PREFIX_ORDER_BY_BIDDER, PREFIX_ORDER_BY_DIRECTION,
    PREFIX_ORDER_BY_PRICE, PREFIX_TICK, remove_tick, read_status, remove_status,
};
use cosmwasm_std::{
    attr, Addr, Attribute, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Event, MessageInfo,
    Order as OrderBy, Response, StdResult, Storage, Uint128,
};

use oraiswap::asset::{pair_key, Asset, AssetInfo};
use oraiswap::error::ContractError;
use oraiswap::limit_order::{
    LastOrderIdResponse, OrderBookMatchableResponse, OrderBookResponse, OrderBooksResponse,
    OrderDirection, OrderFilter, OrderResponse, OrderStatus, OrdersResponse,
};

const RELAY_FEE: u128 = 300u128;
const REWARD_WALLET: &str = "orai16stq6f4pnrfpz75n9ujv6qg3czcfa4qyjux5en";
const ADMIN_WALLET: &str = "orai1tz8wg6kh5su6602h2tmrpnmjlx83xe388nxkn5";

struct Payment {
    address: Addr,
    asset: Asset,
}

pub fn submit_order(
    deps: DepsMut,
    sender: Addr,
    pair_key: &[u8],
    direction: OrderDirection,
    assets: [Asset; 2],
) -> Result<Response, ContractError> {
    if assets[0].amount.is_zero() || assets[1].amount.is_zero() {
        return Err(ContractError::AssetMustNotBeZero {});
    }

    let order_id = increase_last_order_id(deps.storage)?;

    store_order(
        deps.storage,
        &pair_key,
        &Order {
            order_id,
            direction,
            bidder_addr: deps.api.addr_canonicalize(sender.as_str())?,
            offer_amount: assets[0].to_raw(deps.api)?.amount,
            ask_amount: assets[1].to_raw(deps.api)?.amount,
            filled_offer_amount: Uint128::zero(),
            filled_ask_amount: Uint128::zero(),
            status: OrderStatus::Open,
        },
        true,
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "submit_order"),
        (
            "pair",
            &format!("{} - {}", &assets[0].info, &assets[1].info),
        ),
        ("order_id", &order_id.to_string()),
        ("status", &format!("{:?}", OrderStatus::Open)),
        ("direction", &format!("{:?}", direction)),
        ("bidder_addr", sender.as_str()),
        (
            "offer_asset",
            &format!("{} {}", &assets[0].amount, &assets[0].info),
        ),
        (
            "ask_asset",
            &format!("{} {}", &assets[1].amount, &assets[1].info),
        ),
    ]))
}

pub fn cancel_order(
    deps: DepsMut,
    info: MessageInfo,
    order_id: u64,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook_pair = read_orderbook(deps.storage, &pair_key)?;
    let order = read_order(deps.storage, &pair_key, order_id)?;

    if order.bidder_addr != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    // Compute refund asset
    let left_offer_amount = order.offer_amount.checked_sub(order.filled_offer_amount)?;

    let bidder_refund = Asset {
        info: match order.direction {
            OrderDirection::Buy => orderbook_pair.quote_coin_info.to_normal(deps.api)?,
            OrderDirection::Sell => orderbook_pair.base_coin_info.to_normal(deps.api)?,
        },
        amount: left_offer_amount,
    };

    // Build refund msg
    let messages = if left_offer_amount > Uint128::zero() {
        vec![bidder_refund
            .clone()
            .into_msg(None, &deps.querier, deps.api.addr_humanize(&order.bidder_addr)?)?]
    } else {
        vec![]
    };

    remove_order(deps.storage, &pair_key, &order)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "cancel_order"),
        (
            "pair",
            &format!(
                "{} - {}",
                &orderbook_pair.base_coin_info.to_normal(deps.api)?,
                &orderbook_pair.quote_coin_info.to_normal(deps.api)?
            ),
        ),
        ("order_id", &order_id.to_string()),
        ("direction", &format!("{:?}", order.direction)),
        ("status", &format!("{:?}", OrderStatus::Cancel)),
        (
            "bidder_addr",
            &deps.api.addr_humanize(&order.bidder_addr)?.to_string(),
        ),
        ("offer_amount", &order.offer_amount.to_string()),
        ("ask_amount", &order.ask_amount.to_string()),
        ("bidder_refund", &bidder_refund.to_string()),
    ]))
}

pub fn remove_stuff_order(
    deps: DepsMut,
    info: MessageInfo,
    order_id: u64,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook_pair = read_orderbook(deps.storage, &pair_key)?;
    let order = read_order(deps.storage, &pair_key, order_id)?;

    let admin_address = deps.api.addr_canonicalize(ADMIN_WALLET)?;

    if admin_address != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    remove_order(deps.storage, &pair_key, &order)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_stuff_order"),
        (
            "pair",
            &format!(
                "{} - {}",
                &orderbook_pair.base_coin_info.to_normal(deps.api)?,
                &orderbook_pair.quote_coin_info.to_normal(deps.api)?
            ),
        ),
        ("order_id", &order_id.to_string()),
        ("direction", &format!("{:?}", order.direction)),
        ("status", &format!("{:?}", OrderStatus::Cancel)),
        (
            "bidder_addr",
            &deps.api.addr_humanize(&order.bidder_addr)?.to_string(),
        ),
        ("offer_amount", &order.offer_amount.to_string()),
        ("ask_amount", &order.ask_amount.to_string()),
    ]))
}

// pub fn excecute_withdraw(
//     deps: DepsMut,
//     info: MessageInfo,
//     withdraw_asset: Asset,
//     account: Addr,
// ) -> Result<Response, ContractError> {
//     let admin_address = deps.api.addr_canonicalize(ADMIN_WALLET)?;

//     if admin_address != deps.api.addr_canonicalize(info.sender.as_str())? {
//         return Err(ContractError::Unauthorized {});
//     }

//     let bidder_refund = Asset {
//         info: withdraw_asset.info,
//         amount: withdraw_asset.amount,
//     };

//     // Build refund msg
//     let messages = vec![bidder_refund.clone().into_msg(None, &deps.querier, account)?];

//     Ok(Response::new().add_messages(messages).add_attributes(vec![
//         ("action", "withdraw_fund"),
//         ("bidder_refund", &bidder_refund.to_string()),
//     ]))
// }

fn to_events(order: &Order, human_bidder: String, fee: String) -> Event {
    let attrs: Vec<Attribute> = [
        attr("status", format!("{:?}", order.status)),
        attr("bidder_addr", human_bidder),
        attr("order_id", order.order_id.to_string()),
        attr("direction", format!("{:?}", order.direction)),
        attr("offer_amount", order.offer_amount.to_string()),
        attr("filled_offer_amount", order.filled_offer_amount.to_string()),
        attr("ask_amount", order.ask_amount.to_string()),
        attr("filled_ask_amount", order.filled_ask_amount.to_string()),
        attr("fee", fee),
    ]
    .to_vec();
    Event::new("matched_order").add_attributes(attrs)
}

fn process_reward(
    storage: &dyn Storage,
    pair_key: &[u8],
    address: CanonicalAddr,
    reward_assets: [Asset; 2],
) -> Executor {
    let executor = read_reward(storage, &pair_key, &address);
    return match executor {
        Ok(r_reward) => r_reward,
        Err(_err) => Executor::new(address, reward_assets),
    };
}

fn transfer_reward(
    deps: &DepsMut,
    reward: &mut Executor,
    relayer: &mut Executor,
    total_reward: &mut Vec<String>,
    messages: &mut Vec<CosmosMsg>,
)  {
    for i in 0..=1 {
        if Uint128::from(reward.reward_assets[i].amount) >= Uint128::from(1000000u128) {
            messages.push(
                reward.reward_assets[i].into_msg(
                    None,
                    &deps.querier,
                    deps.api
                        .addr_validate(deps.api.addr_humanize(&reward.address).unwrap().as_str()).unwrap(),
                ).unwrap(),
            );
            total_reward.push(reward.reward_assets[i].to_string());
            reward.reward_assets[i].amount = Uint128::zero();
        }

        if Uint128::from(relayer.reward_assets[i].amount) >= Uint128::from(1000000u128) {
            messages.push(
                relayer.reward_assets[i].into_msg(
                    None,
                    &deps.querier,
                    deps.api
                        .addr_validate(deps.api.addr_humanize(&relayer.address).unwrap().as_str()).unwrap(),
                ).unwrap(),
            );
            total_reward.push(relayer.reward_assets[i].to_string());
            relayer.reward_assets[i].amount = Uint128::zero();
        }
    }
}

fn transfer_to_trader(
    deps: &DepsMut,
    list_bidder: Vec<Payment>,
    list_asker: Vec<Payment>,
    messages: &mut Vec<CosmosMsg>,
)  {
    let mut minimalist_asker: Vec<Payment> = vec![];
    let mut minimalist_bidder: Vec<Payment> = vec![];

    for asker in list_asker {
        if let Some(existing_payment) = minimalist_asker
            .iter_mut()
            .find(|p| p.address == asker.address)
        {
            existing_payment.asset.amount += asker.asset.amount;
        } else {
            minimalist_asker.push(asker);
        }
    }
    for bidder in list_bidder {
        if let Some(existing_payment) = minimalist_bidder
            .iter_mut()
            .find(|p| p.address == bidder.address)
        {
            existing_payment.asset.amount += bidder.asset.amount;
        } else {
            minimalist_bidder.push(bidder);
        }
    }

    for asker in minimalist_asker {
        messages.push(asker.asset.into_msg(
            None,
            &deps.querier,
            deps.api.addr_validate(asker.address.as_str()).unwrap(),
        ).unwrap());
    }
    for bidder in minimalist_bidder {
        messages.push(bidder.asset.into_msg(
            None,
            &deps.querier,
            deps.api.addr_validate(bidder.address.as_str()).unwrap(),
        ).unwrap());
    }
}

pub fn excecute_pair(
    deps: DepsMut,
    info: MessageInfo,
    asset_infos: [AssetInfo; 2],
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    // let contract_info = read_config(deps.storage)?;
    let relayer_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let commission_rate = Decimal::from_str("0.001")?;

    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook_pair = read_orderbook(deps.storage, &pair_key)?;

    // let reward_wallet = contract_info.reward_address;
    let reward_wallet = deps.api.addr_canonicalize(REWARD_WALLET)?;
    let reward_assets = [
        Asset {
            info: orderbook_pair.base_coin_info.to_normal(deps.api)?,
            amount: Uint128::zero(),
        },
        Asset {
            info: orderbook_pair.quote_coin_info.to_normal(deps.api)?,
            amount: Uint128::zero(),
        },
    ];
    let mut reward = process_reward(
        deps.storage,
        &pair_key,
        reward_wallet,
        reward_assets.clone(),
    );

    let mut relayer = process_reward(deps.storage, &pair_key, relayer_addr, reward_assets);
    let mut messages: Vec<CosmosMsg> = vec![];

    let mut list_bidder: Vec<Payment> = vec![];
    let mut list_asker: Vec<Payment> = vec![];

    let mut ret_events: Vec<Event> = vec![];
    let mut total_reward: Vec<String> = Vec::new();

    let mut total_orders = 0;
    let mut reward_fee: Uint128;
    let mut relayer_fee: Uint128;
    let (best_buy_price_list, best_sell_price_list) = orderbook_pair
        .find_list_match_price(deps.as_ref().storage, limit)
        .unwrap();

    for buy_price in &best_buy_price_list {
        let mut match_buy_orders = orderbook_pair.find_match_orders(
            deps.as_ref().storage,
            *buy_price,
            OrderDirection::Buy,
            limit,
        ).unwrap();
        for sell_price in &best_sell_price_list {
            if buy_price.lt(sell_price) {
                break;
            }
            let mut match_one_price = false;
            if buy_price.eq(&sell_price) {
                match_one_price = true;
            }

            let mut match_sell_orders = orderbook_pair.find_match_orders(
                deps.as_ref().storage,
                *sell_price,
                OrderDirection::Sell,
                limit,
            ).unwrap();

            for buy_order in &mut match_buy_orders {
                if buy_order
                    .offer_amount
                    .checked_sub(buy_order.filled_offer_amount)?
                    .is_zero()
                    || buy_order.status == OrderStatus::Fulfilled
                {
                    if read_order(deps.storage, &pair_key, buy_order.order_id).is_ok() {
                        remove_order(deps.storage, &pair_key, buy_order)?;
                        ret_events.push(to_events(
                            &buy_order,
                            deps.api.addr_humanize(&buy_order.bidder_addr)?.to_string(),
                            format!("remove stuff order")
                        ));
                    }

                    continue;
                }

                let bidder_addr = deps.api.addr_humanize(&buy_order.bidder_addr)?;
                let mut match_price = buy_order.get_price();
                let mut sell_ask_amount = Uint128::zero();

                for sell_order in &mut match_sell_orders {
                    // check status of sell_order and buy_order
                    let mut lef_sell_offer_amount = sell_order
                        .offer_amount
                        .checked_sub(sell_order.filled_offer_amount)?;
                    let mut lef_buy_offer_amount = buy_order
                        .offer_amount
                        .checked_sub(buy_order.filled_offer_amount)?;

                    if lef_sell_offer_amount.is_zero()
                        || sell_order.status == OrderStatus::Fulfilled
                    {
                        if read_order(deps.storage, &pair_key, sell_order.order_id).is_ok() {
                            remove_order(deps.storage, &pair_key, sell_order)?;
                            ret_events.push(to_events(
                                &sell_order,
                                deps.api.addr_humanize(&sell_order.bidder_addr)?.to_string(),
                                format!("remove stuff order")
                            ));
                        }
                        continue;
                    }

                    if match_one_price == false {
                        if sell_order.order_id < buy_order.order_id {
                            match_price = buy_order.get_price();
                        } else {
                            match_price = sell_order.get_price();
                        }
                    }
                    let mut lef_sell_ask_amount =
                        Uint128::from(lef_sell_offer_amount * match_price);

                    let mut sell_ask_asset = Asset {
                        info: orderbook_pair.quote_coin_info.to_normal(deps.api)?,
                        amount: Uint128::min(lef_buy_offer_amount, lef_sell_ask_amount),
                    };
                    // multiply by decimal atomics because we want to get good round values
                    let sell_offer_amount = Uint128::min(
                        Uint128::from(sell_ask_asset.amount * Decimal::one().atomics())
                            .checked_div(match_price.atomics())
                            .unwrap(),
                        lef_sell_offer_amount,
                    );
                    if sell_ask_asset.amount.is_zero() || sell_offer_amount.is_zero() {
                        continue;
                    }

                    let asker_addr = deps.api.addr_humanize(&sell_order.bidder_addr)?;

                    // fill this order
                    sell_order.fill_order(
                        deps.storage,
                        &pair_key,
                        sell_ask_asset.amount,
                        sell_offer_amount,
                    )?;

                    if !sell_ask_asset.amount.is_zero() {
                        sell_ask_amount = sell_ask_asset.amount;

                        reward_fee = Uint128::min(
                            sell_ask_asset.amount * commission_rate,
                            sell_ask_asset.amount,
                        );

                        reward.reward_assets[1].amount += reward_fee;
                        reward.reward_assets[1].info =
                            orderbook_pair.quote_coin_info.to_normal(deps.api)?;
                        sell_ask_asset.amount = sell_ask_asset.amount.checked_sub(reward_fee)?;

                        relayer_fee = Uint128::min(
                            Uint128::from(RELAY_FEE) * match_price,
                            sell_ask_asset.amount,
                        );

                        relayer.reward_assets[1].amount += relayer_fee;
                        relayer.reward_assets[1].info =
                            orderbook_pair.quote_coin_info.to_normal(deps.api)?;
                        sell_ask_asset.amount = sell_ask_asset.amount.checked_sub(relayer_fee)?;

                        if !sell_ask_asset.amount.is_zero() {
                            let asker_payment: Payment = Payment {
                                address: asker_addr.clone(),
                                asset: Asset {
                                    info: orderbook_pair.quote_coin_info.to_normal(deps.api)?,
                                    amount: sell_ask_asset.amount,
                                },
                            };
                            list_asker.push(asker_payment);
                        }

                        ret_events.push(to_events(
                            &sell_order,
                            deps.api.addr_humanize(&sell_order.bidder_addr)?.to_string(),
                            format!(
                                "{} {}",
                                reward_fee + relayer_fee,
                                &reward.reward_assets[1].info
                            ),
                        ));
                    }

                    // Match with buy order
                    if !sell_offer_amount.is_zero() {
                        buy_order.fill_order(
                            deps.storage,
                            &pair_key,
                            sell_offer_amount,
                            sell_ask_amount,
                        )?;

                        let mut buy_ask_asset = Asset {
                            info: orderbook_pair.base_coin_info.to_normal(deps.api)?,
                            amount: sell_offer_amount,
                        };

                        reward_fee = Uint128::min(
                            buy_ask_asset.amount * commission_rate,
                            buy_ask_asset.amount,
                        );

                        reward.reward_assets[0].amount += reward_fee;
                        reward.reward_assets[0].info =
                            orderbook_pair.base_coin_info.to_normal(deps.api)?;
                        buy_ask_asset.amount = buy_ask_asset.amount.checked_sub(reward_fee)?;

                        relayer_fee = Uint128::min(Uint128::from(RELAY_FEE), buy_ask_asset.amount);

                        relayer.reward_assets[0].amount += relayer_fee;
                        relayer.reward_assets[0].info =
                            orderbook_pair.base_coin_info.to_normal(deps.api)?;
                        buy_ask_asset.amount = buy_ask_asset.amount.checked_sub(relayer_fee)?;

                        if !buy_ask_asset.amount.is_zero() {
                            let bidder_payment: Payment = Payment {
                                address: bidder_addr.clone(),
                                asset: Asset {
                                    info: orderbook_pair.base_coin_info.to_normal(deps.api)?,
                                    amount: buy_ask_asset.amount,
                                },
                            };
                            list_bidder.push(bidder_payment);
                        }

                        ret_events.push(to_events(
                            &buy_order,
                            deps.api.addr_humanize(&buy_order.bidder_addr)?.to_string(),
                            format!(
                                "{} {}",
                                reward_fee + relayer_fee,
                                &reward.reward_assets[0].info
                            ),
                        ));
                    }

                    lef_sell_offer_amount = sell_order
                        .offer_amount
                        .checked_sub(sell_order.filled_offer_amount)?;
                    lef_buy_offer_amount = buy_order
                        .offer_amount
                        .checked_sub(buy_order.filled_offer_amount)?;
                    lef_sell_ask_amount = Uint128::from(lef_sell_offer_amount * match_price);
                    let lef_buy_ask_amount = Uint128::from(
                        lef_buy_offer_amount * Uint128::from(1000000000000000000u128),
                    )
                    .checked_div(match_price * Uint128::from(1000000000000000000u128))
                    .unwrap();

                    if lef_sell_offer_amount > Uint128::zero()
                        && (lef_sell_ask_amount < orderbook_pair.min_quote_coin_amount
                            || lef_sell_ask_amount.is_zero())
                    {
                        reward.reward_assets[0].amount += lef_sell_offer_amount;
                        reward.reward_assets[0].info =
                            orderbook_pair.base_coin_info.to_normal(deps.api)?;

                        sell_order.status = OrderStatus::Fulfilled;
                        remove_order(deps.storage, &pair_key, sell_order)?;

                        ret_events.push(to_events(
                            &sell_order,
                            deps.api.addr_humanize(&sell_order.bidder_addr)?.to_string(),
                            format!(
                                "{} {}",
                                lef_sell_offer_amount, &reward.reward_assets[0].info
                            ),
                        ));
                    }

                    if lef_buy_offer_amount > Uint128::zero()
                        && (lef_buy_offer_amount < orderbook_pair.min_quote_coin_amount
                            || lef_buy_ask_amount.is_zero())
                    {
                        reward.reward_assets[1].amount += lef_buy_offer_amount;
                        reward.reward_assets[1].info =
                            orderbook_pair.quote_coin_info.to_normal(deps.api)?;

                        buy_order.status = OrderStatus::Fulfilled;
                        remove_order(deps.storage, &pair_key, buy_order)?;

                        ret_events.push(to_events(
                            &buy_order,
                            deps.api.addr_humanize(&buy_order.bidder_addr)?.to_string(),
                            format!("{} {}", lef_buy_offer_amount, &reward.reward_assets[1].info),
                        ));
                    }

                    if sell_order.status == OrderStatus::Fulfilled
                        || sell_order.offer_amount == sell_order.filled_offer_amount
                    {
                        total_orders += 1;
                    }
                }
                if buy_order.status == OrderStatus::Fulfilled
                    || buy_order.offer_amount == buy_order.filled_offer_amount
                {
                    total_orders += 1;
                }
            }
        }
    }

    transfer_to_trader(&deps, list_bidder, list_asker, &mut messages);
    transfer_reward(&deps, &mut reward, &mut relayer, &mut total_reward, &mut messages);

    store_reward(deps.storage, &pair_key, &reward)?;
    store_reward(deps.storage, &pair_key, &relayer)?;
    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            ("action", "execute_orderbook_pair"),
            (
                "pair",
                &format!("{} - {}", &asset_infos[0], &asset_infos[1]),
            ),
            ("total_matched_orders", &total_orders.to_string()),
            ("executor_reward", &format!("{:?}", &total_reward)),
        ])
        .add_events(ret_events))
}

pub fn remove_pair(
    deps: DepsMut,
    info: MessageInfo,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {
    let contract_info = read_config(deps.storage)?;
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;

    if contract_info.admin.ne(&sender_addr) {
        return Err(ContractError::Unauthorized {});
    }

    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);

    remove_orderbook(deps.storage, &pair_key);

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_orderbook_pair"),
        (
            "pair",
            &format!("{} - {}", &asset_infos[0], &asset_infos[1]),
        ),
    ]))
}

pub fn execute_remove_price(
    deps: DepsMut,
    info: MessageInfo,
    price: Decimal,
    direction: OrderDirection,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {

    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    
    let admin_address = deps.api.addr_canonicalize(ADMIN_WALLET)?;

    if admin_address != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let total_price = remove_tick(deps.storage, &pair_key, direction, price).unwrap();

    Ok(Response::new().add_attributes(vec![
        ("action", "execute_remove_price"),
        (
            "pair",
            &format!("{} - {}", &asset_infos[0], &asset_infos[1]),
        ),
        (
            "price",
            &format!("{:.5}", &price),
        ),
        (
            "total_price",
            &total_price.to_string(),
        ),
    ]))
}

pub fn execute_remove_status(
    deps: DepsMut,
    info: MessageInfo,
    order_id: u64,
    status: OrderStatus,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {

    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    
    let admin_address = deps.api.addr_canonicalize(ADMIN_WALLET)?;

    if admin_address != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let _ret = remove_status(deps.storage, &pair_key, order_id, status).unwrap();
    
    Ok(Response::new().add_attributes(vec![
        ("action", "execute_remove_status"),
        (
            "pair",
            &format!("{} - {}", &asset_infos[0], &asset_infos[1]),
        ),
        (
            "order_id",
            &order_id.to_string(),
        ),
        (
            "status",
            &format!("{:?}", &status),
        ),
    ]))
}

pub fn query_order(
    deps: Deps,
    asset_infos: [AssetInfo; 2],
    order_id: u64,
) -> StdResult<OrderResponse> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook_pair = read_orderbook(deps.storage, &pair_key)?;
    let order = read_order(deps.storage, &pair_key, order_id)?;

    order.to_response(
        deps.api,
        orderbook_pair.base_coin_info.to_normal(deps.api)?,
        orderbook_pair.quote_coin_info.to_normal(deps.api)?,
    )
}

pub fn query_status(
    deps: Deps,
    asset_infos: [AssetInfo; 2],
    order_id: u64,
    status: OrderStatus,
) -> StdResult<OrderDirection> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let direction = read_status(deps.storage, &pair_key, order_id, status)?;

    Ok(direction)
}

pub fn query_orders(
    deps: Deps,
    asset_infos: [AssetInfo; 2],
    direction: Option<OrderDirection>,
    filter: OrderFilter,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<i32>,
) -> StdResult<OrdersResponse> {
    let order_by = order_by.map_or(None, |val| OrderBy::try_from(val).ok());
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook_pair = read_orderbook(deps.storage, &pair_key)?;

    let (direction_filter, direction_key): (Box<dyn Fn(&OrderDirection) -> bool>, Vec<u8>) =
        match direction {
            // copy value to closure
            Some(d) => (Box::new(move |x| d.eq(x)), d.as_bytes().to_vec()),
            None => (Box::new(|_| true), OrderDirection::Buy.as_bytes().to_vec()),
        };

    let orders: Option<Vec<Order>> = match filter {
        OrderFilter::Bidder(bidder_addr) => {
            let bidder_addr_raw = deps.api.addr_canonicalize(&bidder_addr)?;
            read_orders_with_indexer::<OrderDirection>(
                deps.storage,
                &[
                    PREFIX_ORDER_BY_BIDDER,
                    &pair_key,
                    bidder_addr_raw.as_slice(),
                ],
                direction_filter,
                start_after,
                limit,
                order_by,
            )?
        }
        OrderFilter::Tick {} => read_orders_with_indexer::<u64>(
            deps.storage,
            &[PREFIX_TICK, &pair_key, &direction_key],
            Box::new(|_| true),
            start_after,
            limit,
            order_by,
        )?,
        OrderFilter::Price(price) => {
            let price_key = price.atomics().to_be_bytes();
            read_orders_with_indexer::<OrderDirection>(
                deps.storage,
                &[PREFIX_ORDER_BY_PRICE, &pair_key, &price_key],
                direction_filter,
                start_after,
                limit,
                order_by,
            )?
        }
        OrderFilter::None => match direction {
            Some(_) => read_orders_with_indexer::<OrderDirection>(
                deps.storage,
                &[PREFIX_ORDER_BY_DIRECTION, &pair_key, &direction_key],
                direction_filter,
                start_after,
                limit,
                order_by,
            )?,
            None => Some(read_orders(deps.storage, &pair_key, start_after, limit, order_by)?),
        },
    };

    let resp = OrdersResponse {
        orders: orders.unwrap_or_default()
            .iter()
            .map(|order| {
                order.to_response(
                    deps.api,
                    orderbook_pair.base_coin_info.to_normal(deps.api)?,
                    orderbook_pair.quote_coin_info.to_normal(deps.api)?,
                )
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

pub fn query_orderbooks(
    deps: Deps,
    start_after: Option<Vec<u8>>,
    limit: Option<u32>,
    order_by: Option<i32>,
) -> StdResult<OrderBooksResponse> {
    let order_by = order_by.map_or(None, |val| OrderBy::try_from(val).ok());
    let order_books = read_orderbooks(deps.storage, start_after, limit, order_by)?;
    order_books
        .into_iter()
        .map(|ob| ob.to_response(deps.api))
        .collect::<StdResult<Vec<OrderBookResponse>>>()
        .map(|order_books| OrderBooksResponse { order_books })
}

pub fn query_orderbook(deps: Deps, asset_infos: [AssetInfo; 2]) -> StdResult<OrderBookResponse> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let ob = read_orderbook(deps.storage, &pair_key)?;
    ob.to_response(deps.api)
}

pub fn query_orderbook_is_matchable(
    deps: Deps,
    asset_infos: [AssetInfo; 2],
) -> StdResult<OrderBookMatchableResponse> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let ob = read_orderbook(deps.storage, &pair_key)?;
    let (best_buy_price_list, best_sell_price_list) = ob
        .find_list_match_price(deps.storage, Some(30))
        .unwrap_or_default();

    let mut resp = OrderBookMatchableResponse { is_matchable: true };

    if best_buy_price_list.len() == 0 || best_sell_price_list.len() == 0 {
        resp = OrderBookMatchableResponse {
            is_matchable: false,
        };
    };

    Ok(resp)
}

pub fn query_match_price(
    deps: Deps,
    limit: Option<u32>,
    asset_infos: [AssetInfo; 2],
) -> StdResult<String> {
    let pair_key = pair_key(&[
        asset_infos[0].to_raw(deps.api)?,
        asset_infos[1].to_raw(deps.api)?,
    ]);
    let orderbook = read_orderbook(deps.storage, &pair_key)?;

    let (best_buy_price, best_sell_price) = orderbook.find_match_price(deps.storage, limit).unwrap_or_default();

    let res = format!("best buy price: {:.6} - best sell price: {:.6}", best_buy_price, best_sell_price);

    Ok(res)
}