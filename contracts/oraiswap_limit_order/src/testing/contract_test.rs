use cosmwasm_std::{to_binary, Addr, Coin, Uint128, Decimal};
use oraiswap::create_entry_points_testing;
use oraiswap::testing::{AttributeUtil, MockApp, ATOM_DENOM};

use oraiswap::asset::{Asset, AssetInfo, ORAI_DENOM};
use oraiswap::limit_order::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, LastOrderIdResponse, OrderBooksResponse,
    OrderDirection, OrderFilter, OrderResponse, OrdersResponse, QueryMsg, TicksResponse, OrderBookResponse,
};

use crate::jsonstr;
const USDT_DENOM: &str = "usdt";

#[test]
fn submit_order() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
    ]);

    app.set_token_contract(Box::new(create_entry_points_testing!(oraiswap_token)));

    app.set_token_balances(&[(
        &"asset".to_string(),
        &[(&"addr0000".to_string(), &Uint128::from(1000000000u128))],
    )]);

    let token_addr = app.get_token_addr("asset").unwrap();

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // create order book for pair [orai, usdt]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::from(10u128),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    ).unwrap();

    // Create an existed order book
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(_res);
    
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    // offer asset is null
    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    app.assert_fail(res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(5u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(5u128),
            },
        ],
    };

    // Offer ammount 5 usdt (min 10 usdt) is too low
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(5u128),
            }],
        );
    app.assert_fail(res);

    // paid 150 usdt to get 150 orai
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
        ],
    };

    let _ = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(150u128),
            }],
        )
        .unwrap();

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(0u128),
            },
        ],
    };

    // Asset must not be zero
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(150u128),
            }],
        );
    app.assert_fail(res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    // paid 1000000 orai to get 1000000 atom
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1000000u128),
            }],
        )
        .unwrap();
    println!("submit 2 {:?}", res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(20000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(70000u128),
            },
        ],
    };

    // paid 70000 orai to get 20000 usdt
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(70000u128),
            }],
        )
        .unwrap();
    println!("submit 3 {:?}", res);

    let order_1 = OrderResponse {
        order_id: 1u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(150u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(150u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    let order_2 = OrderResponse {
        order_id: 2u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    let order_3= OrderResponse {
        order_id: 3u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(70000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(20000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Sell,
    };

    assert_eq!(
        order_3.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 3,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    assert_eq!(
        order_2.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 2,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    assert_eq!(
        order_1.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    // create order book for pair [orai, token_addr]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::Token {
            contract_addr: token_addr.clone(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::SubmitOrder {
            direction: OrderDirection::Buy,
            assets: [
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    amount: Uint128::from(1000000u128),
                },
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addr.clone(),
                    },
                    amount: Uint128::from(1000000u128),
                },
            ],
        })
        .unwrap(),
    };

    let _ = app.execute(
        Addr::unchecked("addr0000"), 
        token_addr.clone(), 
        &msg, 
        &[Coin {
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000000u128),
        }],
    )
    .unwrap();
    
    let order_4= OrderResponse {
        order_id: 4u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::Token {
                contract_addr: token_addr.clone(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    assert_eq!(
        order_4.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 4,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::Token {
                        contract_addr: token_addr.clone(),
                    },
                ],
            }
        )
        .unwrap()
    );
    assert_eq!(
        app.query::<LastOrderIdResponse, _>(limit_order_addr.clone(), &QueryMsg::LastOrderId {})
            .unwrap(),
        LastOrderIdResponse { last_order_id: 4 }
    );
}


#[test]
fn update_order() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
    ]);

    app.set_token_contract(Box::new(create_entry_points_testing!(oraiswap_token)));

    app.set_token_balances(&[(
        &"asset".to_string(),
        &[(&"addr0000".to_string(), &Uint128::from(1000000000u128))],
    )]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // update order book for pair [orai, usdt]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::from(10u128),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    // Create an existed order book
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(_res);
    
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    app.assert_fail(res);

    // Offer ammount 5 usdt (min 10 usdt) is too low
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(5u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(5u128),
            },
        ],
    };

    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(5u128),
            }],
        );
    app.assert_fail(res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
        ],
    };

    // paid 150 usdt to get 150 orai
    let _ = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(150u128),
            }],
        )
        .unwrap();

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(150u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(0u128),
            },
        ],
    };

    // Asset must not be zero
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(150u128),
            }],
        );
    app.assert_fail(res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    // paid 1000000 usdt to get 1000000 orai
    let _ = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1000000u128),
            }],
        )
        .unwrap();
    
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(20000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(70000u128),
            },
        ],
    };

    // paid 70000 orai to get 20000 usdt
    let _ = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(70000u128),
            }],
        )
        .unwrap();

    let order_1 = OrderResponse {
        order_id: 1u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(150u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(150u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    let order_2 = OrderResponse {
        order_id: 2u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    let order_3= OrderResponse {
        order_id: 3u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(70000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(20000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Sell,
    };

    assert_eq!(
        order_3.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 3,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    assert_eq!(
        order_2.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 2,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    assert_eq!(
        order_1.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    let update_3 = ExecuteMsg::UpdateOrder {
        order_id: 3u64,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(20000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(50000u128),
            },
        ],
    };

    // paid 50000 orai to get 20000 usdt
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &update_3,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(50000u128),
            }],
        )
        .unwrap();
    println!("update order 3 {:?}", res);

    let update_order_3= OrderResponse {
        order_id: 3u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(50000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(20000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Sell,
    };

    assert_eq!(
        update_order_3.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 3,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    let update_2 = ExecuteMsg::UpdateOrder {
        order_id: 2u64,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(500000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    // paid 500000 usdt to get 1000000 orai
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &update_2,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(500000u128),
            }],
        )
        .unwrap();
    println!("update order 2 {:?}", res);
    
    let update_order_2 = OrderResponse {
        order_id: 2u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(500000u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    assert_eq!(
        update_order_2.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 2,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    let update_1 = ExecuteMsg::UpdateOrder {
        order_id: 1u64,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(50u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(60u128),
            },
        ],
    };

    // paid 50 usdt to get 60 orai
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &update_1,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(50u128),
            }],
        )
        .unwrap();
    println!("update order 1 {:?}", res);

    let update_order_1 = OrderResponse {
        order_id: 1u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(50u128),
            info: AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(60u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    assert_eq!(
        update_order_1.clone(),
        app.query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: USDT_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                ],
            }
        )
        .unwrap()
    );

    let update_4 = ExecuteMsg::UpdateOrder {
        order_id: 1u64,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(50u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(0u128),
            },
        ],
    };

    // Asset must not be zero
    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &update_4,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(50u128),
            }],
        );
    app.assert_fail(res);
    
}

#[test]
fn cancel_order_native_token() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // update order book for pair [orai, atom]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(500000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(500000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(500000u128),
            }],
        )
        .unwrap();

    let msg = ExecuteMsg::CancelOrder {
        order_id: 1,
        asset_infos: [
            AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        ],
    };

    // verfication failed
    let res = app.execute(
        Addr::unchecked("addr0001"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(res);

    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    assert_eq!(
        res.get_attributes(1),
        vec![
            ("action", "cancel_order"),
            ("order_id", "1"),
            ("bidder_refund", &format!("500000{}", USDT_DENOM)),
        ]
    );

    // failed no order exists
    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(res);

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1234560u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1234560u128),
            }],
        )
        .unwrap();

    let msg = ExecuteMsg::CancelOrder {
        order_id: 2,
        asset_infos: [
            AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        ],
    };

    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    assert_eq!(
        res.get_attributes(1),
        vec![
            ("action", "cancel_order"),
            ("order_id", "2"),
            ("bidder_refund", &format!("1234560{}", ORAI_DENOM)),
        ]
    );
}

#[test]
fn cancel_order_token() {
    let mut app = MockApp::new(&[(
        &"addr0000".to_string(),
        &[Coin {
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000000000u128),
        }],
    )]);
    app.set_token_contract(Box::new(create_entry_points_testing!(oraiswap_token)));

    let token_addrs = app.set_token_balances(&[
        (
            &"assetA".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
        (
            &"assetB".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();

    // update order book for pair [token_addrs[0], token_addrs[1]]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[0].clone(),
        },
        quote_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[1].clone(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    // update order book for pair [orai, token_addrs[1]]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[1].clone(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::new(100000u128 ), // Fund must be equal to offer amount
        msg: to_binary(&Cw20HookMsg::SubmitOrder {
            direction: OrderDirection::Buy,
            assets: [
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    amount: Uint128::from(100000u128),
                },
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                    amount: Uint128::from(100000u128),
                },
            ],
        })
        .unwrap(),
    };

    let _ = app.execute(
        Addr::unchecked("addr0000"), 
        token_addrs[0].clone(), 
        &msg, 
        &[],
    )
    .unwrap();

    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::new(100000u128 ),
        msg: to_binary(&Cw20HookMsg::SubmitOrder {
            direction: OrderDirection::Buy,
            assets: [
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    amount: Uint128::from(100000u128),
                },
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                    amount: Uint128::from(100000u128),
                },
            ],
        })
        .unwrap(),
    };

    let _ = app.execute(
        Addr::unchecked("addr0000"), 
        token_addrs[0].clone(), 
        &msg, 
        &[],
    )
    .unwrap();

    let msg = ExecuteMsg::CancelOrder {
        order_id: 1,
        asset_infos: [
            AssetInfo::Token {
                contract_addr: token_addrs[0].clone()
            },
            AssetInfo::Token {
                contract_addr: token_addrs[1].clone(),
            },
        ],
    };

    // failed verfication failed
    let res = app.execute(
        Addr::unchecked("addr0001"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(res);

    let res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    assert_eq!(
        res.get_attributes(1),
        vec![
            ("action", "cancel_order"),
            ("order_id", "1"),
            ("bidder_refund", &format!("100000{}", token_addrs[0])),
        ]
    );

    // failed no order exists
    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(res);
}

#[test]
fn execute_order_native_token() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // update order book for pair [orai, atom]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ATOM_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(6000u128),
            },
        ],
    };

    // offer 6000 atom, ask for 1000 orai
    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ATOM_DENOM.to_string(),
                amount: Uint128::from(6000u128),
            }],
        )
        .unwrap();

    // assertion; native asset balance
    let msg = ExecuteMsg::ExecuteOrder {
        ask_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        offer_asset: Asset {
            amount: Uint128::new(400u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        order_id: 1u64,
    };

    // Native token balance mismatch between the argument and the transferred
    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[Coin {
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000u128),
        }],
    );
    app.assert_fail(res);

    // cannot execute order with other asset
    let res = app.execute(
        Addr::unchecked("addr0001"),
        limit_order_addr.clone(),
        &msg,
        &[Coin {
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(400u128),
        }],
    );
    app.assert_fail(res);

    // partial execute
    let msg = ExecuteMsg::ExecuteOrder {
        ask_info: AssetInfo::NativeToken {
            denom: ATOM_DENOM.to_string(),
        },
        offer_asset: Asset {
            amount: Uint128::new(400u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        order_id: 1u64,
    };
    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(400u128),
            }],
        )
        .unwrap();

    let mut address0_balances = app.query_all_balances(Addr::unchecked("addr0000")).unwrap();
    let mut address1_balances = app.query_all_balances(Addr::unchecked("addr0001")).unwrap();
    println!("round 1 - address0_balances: {:?}", address0_balances);
    println!("round 1 - address1_balances: {:?}", address1_balances);

    let mut expected_balances: Vec<Coin> = [
        Coin{
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(994000u128)
        },
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000400u128),
        }
    ].to_vec();
    assert_eq!(
        address0_balances,
        expected_balances,
    );

    expected_balances = [
        Coin{
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(1002400u128)
        },
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(999600u128),
        }
    ].to_vec();
    assert_eq!(
        address1_balances,
        expected_balances,
    );

    let resp: OrderResponse = app
        .query(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
            },
        )
        .unwrap();
    assert_eq!(resp.filled_ask_amount, Uint128::new(400u128));
    assert_eq!(resp.filled_offer_amount, Uint128::new(2400u128));

    // fill left amount
    let msg = ExecuteMsg::ExecuteOrder {
        ask_info: AssetInfo::NativeToken {
            denom: ATOM_DENOM.to_string(),
        },
        offer_asset: Asset {
            amount: Uint128::new(600u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        order_id: 1u64,
    };
    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(600u128),
            }],
        )
        .unwrap();

    address0_balances = app.query_all_balances(Addr::unchecked("addr0000")).unwrap();
    address1_balances = app.query_all_balances(Addr::unchecked("addr0001")).unwrap();
    println!("round 2 - address0_balances: {:?}", address0_balances);
    println!("round 2 - address1_balances: {:?}", address1_balances);

    expected_balances = [
        Coin{
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(994000u128)
        },
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1001000u128),
        }
    ].to_vec();
    assert_eq!(
        address0_balances,
        expected_balances,
    );

    expected_balances = [
        Coin{
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(1006000u128)
        },
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(999000u128),
        }
    ].to_vec();
    assert_eq!(
        address1_balances,
        expected_balances,
    );

    // no more existed
    assert!(app
        .query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
            }
        )
        .is_err());
}

#[test]
fn execute_order_token() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1000000000u128),
            }],
        ),
        (
            &"addr0001".to_string(),
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1000000u128),
            }],
        ),
    ]);
    app.set_token_contract(Box::new(create_entry_points_testing!(oraiswap_token)));

    let token_addrs = app.set_token_balances(&[
        (
            &"assetA".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
        (
            &"assetB".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();

    // update order book for pair [token_addrs[0]/token_addrs[1]]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[0].clone(),
        },
        quote_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[1].clone(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::SubmitOrder {
            direction: OrderDirection::Buy,
            assets: [
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    amount: Uint128::from(1000000u128),
                },
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                    amount: Uint128::from(1000000u128),
                },
            ],
        })
        .unwrap(),
    };
    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            token_addrs[0].clone(),
            &msg,
            &[],
        )
        .unwrap();

    // cannot execute order with other asset
    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::new(500000u128),
        msg: to_binary(&Cw20HookMsg::ExecuteOrder {
            offer_info: AssetInfo::Token {
                contract_addr: token_addrs[0].clone(),
            },
            order_id: 1u64,
        })
        .unwrap(),
    };
    let res = app.execute(
        Addr::unchecked("addr0001"),
        token_addrs[0].clone(),
        &msg,
        &[],
    );
    // invalid asset given
    app.assert_fail(res);

    // partial execute
    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            token_addrs[1].clone(),
            &msg,
            &[],
        )
        .unwrap();

    let address0_balances = app.query_all_balances(Addr::unchecked("addr0000")).unwrap();
    let address1_balances = app.query_all_balances(Addr::unchecked("addr0001")).unwrap();
    println!("round 1 - address0_balances: {:?}", address0_balances);
    println!("round 1 - address1_balances: {:?}", address1_balances);

    let mut expected_balances: Vec<Coin> = [
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000000000u128),
        }
    ].to_vec();
    assert_eq!(
        address0_balances,
        expected_balances,
    );

    expected_balances = [
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000000u128),
        }
    ].to_vec();
    assert_eq!(
        address1_balances,
        expected_balances,
    );

    let resp: OrderResponse = app
        .query(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                asset_infos: [
                    AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                ],
                order_id: 1,
            },
        )
        .unwrap();

    assert_eq!(resp.filled_ask_amount, Uint128::new(500000u128));
    assert_eq!(resp.filled_offer_amount, Uint128::new(500000u128));

    // fill left amount
    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            token_addrs[1].clone(),
            &msg,
            &[],
        )
        .unwrap(); 

    assert!(app
        .query::<OrderResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Order {
                order_id: 1,
                asset_infos: [
                    AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                ],
            }
        )
        .is_err())
}

#[test]
fn execute_pair_native_token() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0002".to_string(),
            &[
                Coin {
                    denom: USDT_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // Create pair [orai, usdt] for order book
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: USDT_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        precision: None, //Some(Decimal::percent(10)),
        min_base_coin_amount: Uint128::from(10u128),
    };

    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    /* <----------------------------------- order 1 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 2 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(9700u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 3 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(13000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(14000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(13000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 4 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(5000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    // offer usdt, ask for orai
    let _res = app
        .execute(
            Addr::unchecked("addr0002"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(5000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 5 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(4400u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(8800u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0002"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(4400u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 6 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(7000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(14000u128),
            },
        ],
    };

    // offer orai, ask for atom
    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(7000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 7 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(2000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 8 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1200u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1200u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 9 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(5000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 10 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(6789u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(7000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(7000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 11 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1_000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 12 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1600u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 13 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1500u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 14 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1600u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1600u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 15 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 16 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(9700u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 17 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(13000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(14000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(13000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 18 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(5000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
        ],
    };

    // offer usdt, ask for orai
    let _res = app
        .execute(
            Addr::unchecked("addr0002"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(5000u128),
            }],
        )
        .unwrap();
    
    /* <----------------------------------- order 19 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(4400u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(8800u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0002"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(4400u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 20 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(7000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(14000u128),
            },
        ],
    };

    // offer orai, ask for atom
    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(7000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 21 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(2000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 22 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1200u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1200u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 23 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(10000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(5000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(10000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 24 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(6789u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(7000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(7000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 25 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 26 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1600u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 27 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1500u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 28 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1600u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1600u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 29 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(2000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(2000u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 30 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1200u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1200u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 31 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: USDT_DENOM.to_string(),
                },
                amount: Uint128::from(1200u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1500u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: USDT_DENOM.to_string(),
                amount: Uint128::from(1200u128),
            }],
        )
        .unwrap();

    // assertion; native asset balance
    let msg = ExecuteMsg::ExecuteOrderBookPair {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        ],
    };

    // Native token balance mismatch between the argument and the transferred
    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    app.assert_fail(res);

    // Excecute all orders
    let msg = ExecuteMsg::ExecuteOrderBookPair {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: USDT_DENOM.to_string(),
            },
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        ],
    };

    // Native token balance mismatch between the argument and the transferred
    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(960000u128)
    );
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0001"), USDT_DENOM.to_string())
            .unwrap(),
        Uint128::from(960000u128)
    );
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0002"), USDT_DENOM.to_string())
            .unwrap(),
        Uint128::from(981200u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(960000u128)
    );
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0001"), USDT_DENOM.to_string())
            .unwrap(),
        Uint128::from(974800u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(962630u128)
    );
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0001"), USDT_DENOM.to_string())
            .unwrap(),
        Uint128::from(976800u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(964747u128)
    );
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0001"), USDT_DENOM.to_string())
            .unwrap(),
        Uint128::from(978399u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(965160u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(970371u128)
    );

    let _ = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    assert_eq!(
        app.query_balance(Addr::unchecked("addr0000"), ORAI_DENOM.to_string())
            .unwrap(),
        Uint128::from(970373u128)
    );
}

#[test]
fn remove_orderbook_pair() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0002".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };

    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();
    
    // Create pair [orai, atom] for order book
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ATOM_DENOM.to_string(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };

    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );
    
    /* <----------------------------------- order 1 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(12345u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(11111u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ATOM_DENOM.to_string(),
                amount: Uint128::from(11111u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 2 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Sell,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(9700u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(12222u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ATOM_DENOM.to_string(),
                amount: Uint128::from(12222u128),
            }],
        )
        .unwrap();

    /* <----------------------------------- order 3 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(13000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(14000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0001"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(13000u128),
            }],
        )
        .unwrap();
   
    /* <----------------------------------- order 4 -----------------------------------> */
    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1499u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(1900u128),
            },
        ],
    };

    // offer orai, ask for atom
    let _res = app
        .execute(
            Addr::unchecked("addr0002"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1499u128),
            }],
        )
        .unwrap();

    // remove order book for pair [orai, token_addr]
    let msg = ExecuteMsg::RemoveOrderBook {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
            AssetInfo::NativeToken {
                denom: ATOM_DENOM.to_string(),
            },
        ],
    };

    let res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    println!("remove order book pair res: {:?}", res);
    let address0_balances = app.query_all_balances(Addr::unchecked("addr0000")).unwrap();
    let address1_balances = app.query_all_balances(Addr::unchecked("addr0001")).unwrap();
    let address2_balances = app.query_all_balances(Addr::unchecked("addr0002")).unwrap();
    println!("address0_balances: {:?}", address0_balances);
    println!("address1_balances: {:?}", address1_balances);
    println!("address2_balances: {:?}", address2_balances);

    let expected_balances: Vec<Coin> = [
        Coin{
            denom: ATOM_DENOM.to_string(),
            amount: Uint128::from(1000000u128)
        },
        Coin{
            denom: ORAI_DENOM.to_string(),
            amount: Uint128::from(1000000u128),
        }
    ].to_vec();
    assert_eq!(
        address0_balances,
        expected_balances,
    );
    assert_eq!(
        address1_balances,
        expected_balances,
    );
    assert_eq!(
        address2_balances,
        expected_balances,
    );
}

#[test]
fn orders_querier() {
    let mut app = MockApp::new(&[
        (
            &"addr0000".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
        (
            &"addr0001".to_string(),
            &[
                Coin {
                    denom: ORAI_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: ATOM_DENOM.to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
        ),
    ]);

    app.set_token_contract(Box::new(create_entry_points_testing!(oraiswap_token)));

    let token_addrs = app.set_token_balances(&[
        (
            &"assetA".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
        (
            &"assetB".to_string(),
            &[
                (&"addr0000".to_string(), &Uint128::from(1000000000u128)),
                (&"addr0001".to_string(), &Uint128::from(1000000000u128)),
            ],
        ),
    ]);

    let msg = InstantiateMsg {
        name: None,
        version: None,
        admin: None,
    };
    let code_id = app.upload(Box::new(create_entry_points_testing!(crate)));
    let limit_order_addr = app
        .instantiate(
            code_id,
            Addr::unchecked("addr0000"),
            &msg,
            &[],
            "limit order",
        )
        .unwrap();

    // update order book for pair [orai, atom]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::NativeToken {
            denom: ORAI_DENOM.to_string(),
        },
        quote_coin_info: AssetInfo::NativeToken {
            denom: ATOM_DENOM.to_string(),
        },
        precision: Some(Decimal::percent(10)),
        min_base_coin_amount: Uint128::from(10u128),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    // update order book for pair [token_addrs[0], token_addrs[1]]
    let msg = ExecuteMsg::CreateOrderBookPair {
        base_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[0].clone(),
        },
        quote_coin_info: AssetInfo::Token {
            contract_addr: token_addrs[1].clone(),
        },
        precision: None,
        min_base_coin_amount: Uint128::zero(),
    };
    let _res = app.execute(
        Addr::unchecked("addr0000"),
        limit_order_addr.clone(),
        &msg,
        &[],
    );

    // query orderbooks
    let res = app
        .query::<OrderBookResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::OrderBook {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ]
            },
        )
        .unwrap();
    println!("[LOG] 1st orderbooks :{}", jsonstr!(res));

    // query all orderbooks
    let res = app
        .query::<OrderBooksResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::OrderBooks {
                start_after: None,
                limit: None,
                order_by: None,
            },
        )
        .unwrap();

    println!("orderbooks :{}", jsonstr!(res));

    let msg = ExecuteMsg::SubmitOrder {
        direction: OrderDirection::Buy,
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: Uint128::from(1000000u128),
            },
        ],
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            limit_order_addr.clone(),
            &msg,
            &[Coin {
                denom: ORAI_DENOM.to_string(),
                amount: Uint128::from(1000000u128),
            }],
        )
        .unwrap();

    // user sends token therefore no need to set allowance for limit order contract
    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: limit_order_addr.to_string(),
        amount: Uint128::from(1000000u128),
        msg: to_binary(&Cw20HookMsg::SubmitOrder {
            direction: OrderDirection::Buy,
            assets: [
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    amount: Uint128::from(1000000u128),
                },
                Asset {
                    info: AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                    amount: Uint128::from(1000000u128),
                },
            ],
        })
        .unwrap(),
    };

    let _res = app
        .execute(
            Addr::unchecked("addr0000"),
            token_addrs[0].clone(),
            &msg,
            &[],
        )
        .unwrap();

    let order_1 = OrderResponse {
        order_id: 1u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ORAI_DENOM.to_string(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::NativeToken {
                denom: ATOM_DENOM.to_string(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    let order_2 = OrderResponse {
        order_id: 2u64,
        bidder_addr: "addr0000".to_string(),
        offer_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::Token {
                contract_addr: token_addrs[0].clone(),
            },
        },
        ask_asset: Asset {
            amount: Uint128::from(1000000u128),
            info: AssetInfo::Token {
                contract_addr: token_addrs[1].clone(),
            },
        },
        filled_offer_amount: Uint128::zero(),
        filled_ask_amount: Uint128::zero(),
        direction: OrderDirection::Buy,
    };

    assert_eq!(
        OrdersResponse {
            orders: vec![order_2.clone(),],
        },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                ],
                direction: None,
                filter: OrderFilter::Bidder("addr0000".to_string()),
                start_after: None,
                limit: None,
                order_by: Some(1),
            }
        )
        .unwrap()
    );

    assert_eq!(
        OrdersResponse {
            orders: vec![order_1.clone()],
        },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
                direction: None,
                filter: OrderFilter::None,
                start_after: None,
                limit: None,
                order_by: Some(1),
            }
        )
        .unwrap()
    );

    // DESC test
    assert_eq!(
        OrdersResponse {
            orders: vec![order_2.clone()],
        },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::Token {
                        contract_addr: token_addrs[0].clone(),
                    },
                    AssetInfo::Token {
                        contract_addr: token_addrs[1].clone(),
                    },
                ],
                direction: None,
                filter: OrderFilter::None,
                start_after: None,
                limit: None,
                order_by: Some(2),
            }
        )
        .unwrap()
    );

    // different bidder
    assert_eq!(
        OrdersResponse { orders: vec![] },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
                direction: None,
                filter: OrderFilter::Bidder("addr0001".to_string()),
                start_after: None,
                limit: None,
                order_by: None,
            }
        )
        .unwrap()
    );

    // start after DESC
    assert_eq!(
        OrdersResponse {
            orders: vec![order_1],
        },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
                direction: None,
                filter: OrderFilter::None,
                start_after: Some(2u64),
                limit: None,
                order_by: Some(2),
            }
        )
        .unwrap()
    );

    // start after ASC
    assert_eq!(
        OrdersResponse { orders: vec![] },
        app.query::<OrdersResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Orders {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
                direction: None,
                filter: OrderFilter::None,
                start_after: Some(1u64),
                limit: None,
                order_by: Some(1),
            }
        )
        .unwrap()
    );

    // query all ticks
    let res = app
        .query::<TicksResponse, _>(
            limit_order_addr.clone(),
            &QueryMsg::Ticks {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: ORAI_DENOM.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: ATOM_DENOM.to_string(),
                    },
                ],
                direction: OrderDirection::Buy,
                start_after: None,
                limit: None,
                order_by: Some(1),
            },
        )
        .unwrap();
    println!("[HIEU_LOG]: ticks res: {:?}\n\n\n", res);
    for tick in res.ticks {
        let res = app
            .query::<OrdersResponse, _>(
                limit_order_addr.clone(),
                &QueryMsg::Orders {
                    asset_infos: [
                        AssetInfo::NativeToken {
                            denom: ORAI_DENOM.to_string(),
                        },
                        AssetInfo::NativeToken {
                            denom: ATOM_DENOM.to_string(),
                        },
                    ],
                    direction: None,
                    filter: OrderFilter::Price(tick.price),
                    start_after: None,
                    limit: None,
                    order_by: Some(1),
                },
            )
            .unwrap();
        println!("{:?}", res);
    }
}
