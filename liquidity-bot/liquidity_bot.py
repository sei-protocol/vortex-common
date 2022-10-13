import argparse
import json
from os import truncate
import random
import subprocess
import sys
import time
import random
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
SEND_FUND_TMPL = (
    " seid tx bank send -y admin {account} 1000000000000factory/sei1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqpeheyc/uust2"
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
        quantity = truncate(random.uniform(0, 1) * 10, 1)
        print("quantity", quantity)
        if order_type == "SHORT":
            price = self.get_oracle_price() + truncate(random.uniform(0, 1) * 5, 1)
        else:
            price = self.get_oracle_price() - truncate(random.uniform(0, 1) * 5, 1)

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

    def send_fund(self):
        print("send fund")
        result = subprocess.check_output(
            [
                CMD.format(password=self.password, binary=self.binary) + 
                SEND_FUND_TMPL.format(
                    key=self.key, 
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

    accounts = open(str(Path.home()) + 'accounts.txt', 'r')
    
    while True:
        account = accounts.readline()
        if not account:
            break

        lb.key = account
        side = random.random()
        if side < 0.33 or side > 0.66:
            lb.place_order("LONG")
        if side > 0.33:
            lb.place_order("SHORT")
        time.sleep(30)
