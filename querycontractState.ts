import { SimulateCosmWasmClient } from '@oraichain/cw-simulate';

import fs from 'fs';

const downloadState = async (contractAddress, writeCallback, endCallback, startAfter?: string, limit = 1000) => {
  let nextKey = startAfter;

  while (true) {
    const url = new URL(`https://lcd.orai.io/cosmwasm/wasm/v1/contract/${contractAddress}/state`);
    url.searchParams.append('pagination.limit', limit.toString());
    if (nextKey) {
      url.searchParams.append('pagination.key', nextKey);
      console.log('nextKey', nextKey);
    }
    try {
      const { models, pagination } = await fetch(url.toString(), { signal: AbortSignal.timeout(30000) }).then((res) =>
        res.json()
      );
      writeCallback(models);
      if (!(nextKey = pagination.next_key)) {
        return endCallback();
      }
    } catch (ex) {
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
};

const saveState = async (contractAddress: string, nextKey?: string) => {
  await new Promise((resolve) => {
    downloadState(
      contractAddress,
      (chunks) => {
        const data = chunks.map(({ key, value }) => `${Buffer.from(key, 'hex').toString('base64')},${value}`);
        fs.appendFileSync(`${contractAddress}.csv`, data.join('\n') + '\n');
      },
      resolve,
      nextKey
    );
  });

  const {
    contract_info: { code_id }
  } = await fetch(`https://lcd.orai.io/cosmwasm/wasm/v1/contract/${contractAddress}`).then((res) => res.json());
  const { data } = await fetch(`https://lcd.orai.io/cosmwasm/wasm/v1/code/${code_id}`).then((res) => res.json());
  fs.writeFileSync(contractAddress, Buffer.from(data, 'base64'));

  console.log('done');
};

const loadState = async (contractAddress: string) => {
  const data = fs
    .readFileSync(`${contractAddress}.csv`)
    .toString()
    .trim()
    .split('\n')
    .map((line) => line.split(',', 2));

  return data;
};

const client = new SimulateCosmWasmClient({
  chainId: 'Oraichain',
  bech32Prefix: 'orai',
  metering: true
});
const senderAddress = 'orai1tz8wg6kh5su6602h2tmrpnmjlx83xe388nxkn5';

(async () => {
  const contractAddress = 'orai1ase8wkkhczqdda83f0cd9lnuyvf47465j70hyk';
  // saveState(contractAddress);

  console.time('start');
  const data = await loadState(contractAddress);
  console.timeEnd('start');
  const { codeId } = await client.upload(senderAddress, fs.readFileSync(contractAddress), 'auto');
  await client.loadContract(
    contractAddress,
    {
      codeId,
      admin: senderAddress,
      label: 'limit order contract',
      creator: senderAddress,
      created: 1
    },
    data
  );

  const ret = await client.queryContractSmart(contractAddress, { all_tokens: {} });
  console.log(ret);
})();