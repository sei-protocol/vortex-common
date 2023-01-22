# Vortex Liquidity Bot 

This script provides a template to send LONG & SHORT orders to a vortex contract.

## User Manual
1. Change the price denom to the designated denom (e.g. UST2)
2. Start the process with the systemd configuration below
```
[Unit]
Description=Liquidity bot service
After=multi-user.target
[Service]
Type=simple
Restart=always
Environment="HOME=/root"
ExecStart=/usr/bin/python3 /home/ubuntu/vortex-common/liquidity-bot/liquidity_bot.py $KEY $PASSWORD $CONTRACT_ADDR $CHAIN_ID --binary /root/go/bin/seid
[Install]
WantedBy=multi-user.target
```
3. Check the log with journalctl to confirm the orders are sent successfully