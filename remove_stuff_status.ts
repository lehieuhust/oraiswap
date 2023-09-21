import { ExecuteInstruction, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { stringToPath } from "@cosmjs/crypto";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import dotenv from 'dotenv';
dotenv.config();

export type UserWallet = { address: string; client: SigningCosmWasmClient };

async function delay(timeout: number) {
  return new Promise((resolve) => setTimeout(resolve, timeout));
}

export async function setupWallet(mnemonic: string): Promise<UserWallet> {
  const prefix = "orai";
  
  if (!mnemonic || mnemonic.length < 48) {
    throw new Error("Must set MNEMONIC to a 12 word phrase");
  }
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    hdPaths: [stringToPath(process.env.HD_PATH || "m/44'/118'/0'/0/0")],
    prefix
  });
  const [firstAccount] = await wallet.getAccounts();
  const address = firstAccount.address;
  const client = await SigningCosmWasmClient.connectWithSigner(process.env.RPC_URL!, wallet, {
    gasPrice: GasPrice.fromString("0.002orai")
  });

  return { address, client };
}

export const removeStatus = async (
  contractAddr: string,
  sender: UserWallet,
  index: number,
): Promise<number> => {
  const multipleremoveStatus: ExecuteInstruction[] = [];
  const assetInfos = [
    {
      native_token: {
        denom: "orai"
      }
    },
    {
      token: {
        contract_addr: "orai12hzjxfh77wl572gdzct2fxv2arxcwh6gykc7qh"
      }
    }
  ];
  console.log("current index: ");
  console.log({index});
  for (let i = index; i < index + 1000; i++) {
    const statusDirection = {
      status: {
        asset_infos: assetInfos,
        order_id: i,
        status: "open"
      }
    };

    try {
      const status_direction = await sender.client.queryContractSmart(contractAddr, statusDirection);
      console.log({i});
      console.log({status_direction});

      const removeStatus: ExecuteInstruction = {
        contractAddress: contractAddr,
        msg: {
          remove_status: {
            asset_infos: assetInfos,
            order_id: i,
            status: "open"
          }
        }
      };
      multipleremoveStatus.push(removeStatus);

    } catch (error) {
      console.log("status of order_id: ", i, "is not found")
      console.error(error);
    }
    // const removeStatus: ExecuteInstruction = {
    //   contractAddress: contractAddr,
    //   msg: {
    //     remove_status: {
    //       asset_infos: assetInfos,
    //       order_id: i,
    //       status: "open"
    //     }
    //   }
    // };
    // multipleremoveStatus.push(removeStatus);
  }
  // if (multipleremoveStatus.length > 0) {
  //   try {
  //     const removeStatusResult = await sender.client.executeMultiple(sender.address, multipleremoveStatus, "auto");
  //     console.log("remove status - txHash:", removeStatusResult.transactionHash);
  //   } catch (error) {
  //     console.error(error);
  //   }
  // }
  return index + 1000;
}

(async () => {
  let mnemonic = process.env.MNEMONIC;
  const contractAddr = "orai1nt58gcu4e63v7k55phnr3gaym9tvk3q4apqzqccjuwppgjuyjy6sxk8yzp";
  const admin = await setupWallet(mnemonic as string);

  
  // let index = 1190973
  let index = 1106572
  while (index<1957828) {
    const { amount } = await admin.client.getBalance(admin.address, "orai");
    console.log(`balance of ${admin.address} is ${amount}`);
    if (parseInt(amount) <= 2000000) {
      throw new Error(
        `Balance(${amount}) of ${admin.address} must be greater than 1 ORAI`
      );
    }
    try {
      index = await removeStatus(contractAddr, admin, index);
      console.log("after remove status");
      console.log({index});
    } catch (error) {
      console.error(error);
    }
    
    await delay(10000);
  }
})();