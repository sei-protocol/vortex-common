import argparse
import json
import random
import subprocess
import sys
import time
from pathlib import Path

base_price = 0
quantity_floor = 0
quantity_ceiling = 0

CMD = "printf '{password}\n' | {binary}"
PLACE_ORDER_TMPL = (
    " tx dex place-orders {contract} "
    "\'{order_type}?{price}?{quantity}?{price_denom}?{asset_denom}?LIMIT?{order_data}\'"
    " --amount=100000000uusdc -y --from={key} --chain-id={chain_id} --fees=1000000usei --gas=50000000 --broadcast-mode=block"
)
EXCHANGE_RATES_TMPL = (
    " query oracle exchange-rates | grep -A 2 \"{denom}\" | "
    "sed -n 's/.*exchange_rate: \"//p' | sed -n 's/\"//p'"
 )

class OrderData:
    def __init__(self, position_effect="Open", leverage="1") -> None:
        self.position_effect = position_effect
        self.leverage = leverage

class LiquidityBot:
    def __init__(self, key, password, contract, chain_id, binary) -> None:
        self.key = key
        self.password = password
        self.contract = contract
        self.binary = binary
        self.chain_id = chain_id

    def generate_random_order(account_address):
        ot = "Long"
        scale = ((3*(random.uniform(0, 1)**3)+1)**2)/50
        p = round(base_price * (1 + scale), 2)
        l = 1
        q = random.randint(quantity_floor, quantity_ceiling)

        if random.getrandbits(1):
            ot = "Short"
            p = round(base_price * (1 - scale), 2)
            l += 1

        return PLACE_ORDER.format(contract=contract_address,
                                account=account_address,
                                order_type=ot,
                                price=p,
                                quantity=q,
                                price_denom="SEI",
                                asset_denom="ATOM",
                                leverage=l,
                                amount=round(p * q * 1000500)
                                )

    def get_oracle_price(self):
        result = subprocess.check_output(
            [
                CMD.format(password=self.password, binary=self.binary) + 
                EXCHANGE_RATES_TMPL.format(
                    denom="uatom"
                )
            ],
            stderr=subprocess.STDOUT,
            shell=True,
        )

        return float(result.decode('utf-8'))

    def place_order(self, order_type):
        print("place_order")
        od = OrderData()
        data_json = json.dumps(OrderData().__dict__)

        # randomize order price and quantity
        quantity = round(random.uniform(0, 1) * 10, 2)
        print("quantity", quantity)
        if order_type == "SHORT":
            price = self.get_oracle_price() + round(random.uniform(0, 1) * 5, 2)
        else:
            price = self.get_oracle_price() - round(random.uniform(0, 1) * 5, 2)

        result = subprocess.check_output(
            [
                CMD.format(password=self.password, binary=self.binary) + 
                PLACE_ORDER_TMPL.format(
                    contract=self.contract,
                    order_type=order_type,
                    price=price,
                    quantity=quantity,
                    price_denom="USDC",
                    asset_denom="ATOM",
                    key=self.key, 
                    chain_id=self.chain_id, 
                    order_data=data_json
                ) 
            ],
            stderr=subprocess.STDOUT,
            shell=True,
        )
        print(result)


if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("key", help='Your wallet (key) name', type=str)
    parser.add_argument("password", help='The keychain password', type=str)
    parser.add_argument('contract', help='Contract address', type=str)
    parser.add_argument('chain_id', help='Chain id', type=str, default='sei-chain')
    parser.add_argument('--binary', help='Your seid binary path', type=str, default=str(Path.home()) + '/go/bin/seid')
    args=parser.parse_args()

    lb = LiquidityBot(args.key, args.password, args.contract, args.chain_id, args.binary)
    
    while True:
        lb.place_order("LONG")
        lb.place_order("SHORT")
        time.sleep(30)
