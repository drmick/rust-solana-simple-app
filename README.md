# Solana program: Index

The project consists of three modules
1. [solana-program](program) (Smart Contract) 
2. [price-sender](client/src/price_sender) - module that adds the BTC price to solana-program
2. [client](client/src/client) - solana-program client module (here we can see the average bitcoin price)


## Startup steps

#### 1. Configure solana to work on the local network or in devnet and add some SOL
```
solana config set --url https://api.devnet.solana.com
solana airdrop 2
solana airdrop 2
```

#### 2. Build solana-program and generate keypair of program: 
```
./run.sh build-bpf
```

#### 3. Deploy solana-program:
```
./run.sh deploy
```

#### 4. Run bitcoin price sender to solana-program:
```
./run.sh price_sender
```

#### 5 Run client:
```
./run.sh client
```



[Here](https://explorer.solana.com/address/5hWnTmjBFCsTwxJTztNNGXHznBze42N8XReshWnWxubQ?cluster=devnet) is example of program (with running price-sender)
