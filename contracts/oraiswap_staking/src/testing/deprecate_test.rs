use crate::contract::{execute, instantiate, query};
use crate::state::{read_pool_info, store_pool_info};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, Api, Decimal, StdError, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use oraiswap::asset::{Asset, AssetInfo, ORAI_DENOM};
use oraiswap::staking::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, PoolInfoResponse, QueryMsg, RewardInfoResponse,
    RewardInfoResponseItem,
};
use oraiswap::testing::ATOM_DENOM;

#[test]
fn test_deprecate() {
    let mut deps = mock_dependencies_with_balance(&[
        coin(10000000000u128, ORAI_DENOM),
        coin(20000000000u128, ATOM_DENOM),
    ]);

    let msg = InstantiateMsg {
        owner: Some(Addr::unchecked("owner")),
        rewarder: Addr::unchecked("rewarder"),
        minter: Some(Addr::unchecked("mint")),
        oracle_addr: Addr::unchecked("oracle"),
        factory_addr: Addr::unchecked("factory"),
        base_denom: None,
    };

    let info = mock_info("addr", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::RegisterAsset {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("asset"),
        },
        staking_token: Addr::unchecked("staking"),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let asset_key = deps.api.addr_canonicalize("asset").unwrap();
    let pool_info = read_pool_info(&deps.storage, &asset_key).unwrap();
    store_pool_info(&mut deps.storage, &asset_key, &pool_info).unwrap();

    // set rewards per second for asset
    // will also add to the index the pending rewards from before the migration
    let msg = ExecuteMsg::UpdateRewardsPerSec {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("asset"),
        },
        assets: vec![
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ORAI_DENOM.to_string(),
                },
                amount: 100u128.into(),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: ATOM_DENOM.to_string(),
                },
                amount: 200u128.into(),
            },
        ],
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // bond 100 tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {
            asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
        })
        .unwrap(),
    });
    let info = mock_info("staking", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // owner of reward contract deposit 100 reward tokens
    // distribute weight => 80:20
    let msg = ExecuteMsg::DepositReward {
        rewards: vec![Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
            amount: Uint128::from(100u128),
        }],
    };
    let info = mock_info("rewarder", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // query pool and reward info
    let res: PoolInfoResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::PoolInfo {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset"),
                },
            },
        )
        .unwrap(),
    )
    .unwrap();
    let res_cmp = res.clone();
    assert_eq!(
        res_cmp,
        PoolInfoResponse {
            total_bond_amount: Uint128::from(100u128),
            reward_index: Decimal::from_ratio(100u128, 100u128),
            migration_index_snapshot: None,
            migration_deprecated_staking_token: None,
            ..res
        }
    );
    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::RewardInfo {
            asset_info: None,
            // asset_info: Some(AssetInfo::Token {
            //     contract_addr: Addr::unchecked("asset"),
            // }),
            staker_addr: Addr::unchecked("addr"),
        },
    )
    .unwrap();
    let res: RewardInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        RewardInfoResponse {
            staker_addr: Addr::unchecked("addr"),
            reward_infos: vec![RewardInfoResponseItem {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset"),
                },
                bond_amount: Uint128::from(100u128),
                pending_reward: Uint128::from(100u128),
                pending_withdraw: vec![],
                should_migrate: None,
            }],
        }
    );

    // execute deprecate
    let msg = ExecuteMsg::DeprecateStakingToken {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("asset"),
        },
        new_staking_token: Addr::unchecked("new_staking"),
    };
    let info = mock_info("owner", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // deposit more rewards
    let msg = ExecuteMsg::DepositReward {
        rewards: vec![Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
            amount: Uint128::from(100u128),
        }],
    };
    let info = mock_info("rewarder", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // query again
    let res: PoolInfoResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::PoolInfo {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset"),
                },
            },
        )
        .unwrap(),
    )
    .unwrap();
    let res_cmp = res.clone();
    assert_eq!(
        res_cmp,
        PoolInfoResponse {
            staking_token: Addr::unchecked("new_staking"),
            total_bond_amount: Uint128::zero(), // reset
            reward_index: Decimal::from_ratio(100u128, 100u128), // stays the same
            migration_index_snapshot: Some(Decimal::from_ratio(100u128, 100u128)),
            migration_deprecated_staking_token: Some(Addr::unchecked("staking")),
            pending_reward: Uint128::from(100u128), // new reward waiting here
            ..res
        }
    );
    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::RewardInfo {
            asset_info: None,
            staker_addr: Addr::unchecked("addr"),
        },
    )
    .unwrap();
    let res: RewardInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        RewardInfoResponse {
            staker_addr: Addr::unchecked("addr"),
            reward_infos: vec![RewardInfoResponseItem {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset")
                },
                bond_amount: Uint128::from(100u128),
                pending_reward: Uint128::from(100u128), // did not change
                pending_withdraw: vec![],
                should_migrate: Some(true), // non-short pos should migrate
            }],
        }
    );

    // try to bond new or old staking token, should fail both
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {
            asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
        })
        .unwrap(),
    });
    let info = mock_info("staking", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("The staking token for this asset has been migrated to new_staking")
    );
    let info = mock_info("new_staking", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("The LP token for this asset has been deprecated, withdraw all your deprecated tokens to migrate your position")
    );

    // unbond all the old tokens
    let msg = ExecuteMsg::Unbond {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("asset"),
        },
        amount: Uint128::from(100u128),
    };
    let info = mock_info("addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    // make sure that we are receiving deprecated lp tokens tokens
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "staking".into(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        })]
    );
    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::RewardInfo {
            asset_info: None,
            staker_addr: Addr::unchecked("addr"),
        },
    )
    .unwrap();
    let res: RewardInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        RewardInfoResponse {
            staker_addr: Addr::unchecked("addr"),
            reward_infos: vec![RewardInfoResponseItem {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset")
                },
                bond_amount: Uint128::zero(),
                pending_reward: Uint128::from(100u128), // still the same
                pending_withdraw: vec![],
                should_migrate: None, // now its back to empty
            },],
        }
    );

    // now can bond the new staking token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {
            asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
        })
        .unwrap(),
    });
    let info = mock_info("new_staking", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // deposit new rewards
    // will also add to the index the pending rewards from before the migration
    let msg = ExecuteMsg::DepositReward {
        rewards: vec![Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
            amount: Uint128::from(100u128),
        }],
    };
    let info = mock_info("rewarder", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // expect to have 80 * 3 rewards
    // initial + deposit after deprecation + deposit after bonding again
    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::RewardInfo {
            asset_info: None,
            staker_addr: Addr::unchecked("addr"),
        },
    )
    .unwrap();
    let res: RewardInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        RewardInfoResponse {
            staker_addr: Addr::unchecked("addr"),
            reward_infos: vec![RewardInfoResponseItem {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset")
                },
                bond_amount: Uint128::from(100u128),
                pending_reward: Uint128::from(300u128), // 100 * 3
                pending_withdraw: vec![],
                should_migrate: None,
            },],
        }
    );

    // completely new users can bond
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "newaddr".into(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {
            asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
        })
        .unwrap(),
    });
    let info = mock_info("new_staking", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::RewardInfo {
            asset_info: None,
            staker_addr: Addr::unchecked("newaddr"),
        },
    )
    .unwrap();
    let res: RewardInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        RewardInfoResponse {
            staker_addr: Addr::unchecked("newaddr"),
            reward_infos: vec![RewardInfoResponseItem {
                asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset")
                },
                bond_amount: Uint128::from(100u128),
                pending_reward: Uint128::zero(),
                pending_withdraw: vec![],
                should_migrate: None,
            },],
        }
    );
}
