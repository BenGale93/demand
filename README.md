# Demand

A tool for dynamically pricing goods based on their demand.

## Architecture

A central server (pricer) contains the valid list of goods and their
properties. It will provide a list of these goods and their current price to
any listener.

A listener will display the prices to a board or website.

Point of Sale (PoS) units will be able to connect to the pricer and broadcast
sales to it. Over a given time period, the pricer will consume the given
purchases and reprice the items.
