# Oraiswap Router <!-- omit in toc -->

The Router Contract contains the logic to facilitate multi-hop swap operations via native & oraiswap.

**On-chain swap & Oraiswap is supported.**

Columbus-4 Contract:

- https://finder.orai.money/columbus-4/address/orai19qx5xe6q9ll4w0890ux7lv2p4mf3csd4qvt3ex

Tequila-0004 Contract:

- https://finder.orai.money/tequila-0004/address/orai14z80rwpd0alzj4xdtgqdmcqt9wd9xj5ffd60wp

Tx:

- KRT => UST => mABNB: https://finder.orai.money/tequila-0004/tx/46A1C956D2F4F7A1FA22A8F93749AEADB953ACDFC1B9FB7661EEAB5C59188175
- mABNB => UST => KRT: https://finder.orai.money/tequila-0004/tx/E9D63CE2C8AC38F6C9434C62F9A8B59F38259FEB86F075D43C253EA485D7F0A9

### Operations Assertion

The contract will check whether the resulting token is swapped into one token.

### Example

Swap KRT => UST => mABNB

```
{
   "handle_swap_operations":{
      "operations":[
         {
            "native_swap":{
               "offer_denom":"ukrw",
               "ask_denom":"uusd"
            }
         },
         {
            "orai_swap":{
               "offer_asset_info":{
                  "native_token":{
                     "denom":"uusd"
                  }
               },
               "ask_asset_info":{
                  "token":{
                     "contract_addr":"orai1avryzxnsn2denq7p2d7ukm6nkck9s0rz2llgnc"
                  }
               }
            }
         }
      ],
      "minimum_receive":"88000"
   }
}
```