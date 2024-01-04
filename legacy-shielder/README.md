# Shielder

This repository contains an implementation of Shielder: a construct allowing you to hold assets and use them privately, without revealing to the outside world the exact papertrail of your transactions. As such, you can think of Shielder in its core as a private wallet.

Shielder is a part of, and a first project within the [Liminal](https://alephzero.org/ecosystem/liminal) privacy framework.

What you will find here is not only the smart contract implementing the Shielder functionality (in conjunction with the [Aleph Zero](https://alephzero.org) blockchain, which facilitates certain privacy-related operations) but also a handful of utilities, including a basic PSP22 token implementaion. These allow you to quickly bootstrap a local chain with Shielder already deployed and get straight to experimentation.
Of course, you can choose to connect to the #Smarknet chain on Aleph Zero, but for getting familiar with the concepts and experimentation we strongly recommend using a local chain.

## Installation

Bootstrapping a working instance of Shielder is easy as π.

There are some basic dependencies you need to have on your machine before proceeding:
* docker (click [here](https://docs.docker.com/engine/install/) for installation instructions)
* docker-compose (for now this repo uses the standalone version, click [here](https://docs.docker.com/compose/install/other/) for installation instructions)
* jq ([installation instructions](https://lindevs.com/install-jq-on-ubuntu))
* Rust (https://www.rust-lang.org/tools/install)

With that out of the way, all you need to do is run:
```bash
./deploy/deploy.sh
```

After a few minutes and several screens of logs, you should have everything installed. Note that as a last step, the script will ask you to provide a password to encrypt the shielder state. If you are just experimenting on your local machine, you can just press 'Enter' to use an empty password.

## Usage

First of all, it's worth noting that we already have pretty much everything set up for us:
* two PSP22 token contract instances (you can learn their addresses by inspecting the `deploy/accounts.json` file: they will come in handy!)
* a Shielder contract instance (its address is also a part of `deploy/accounts.json`)
* two accounts: Damian and Hans, with addresses `5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH` and `5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o`, respectively (inspect the `deploy/deploy.sh` if the ones pasted here don't work for you).
* there's also a third account, Alice, which has admin rights. That said, for the purpose of this tutorial we'll stick to using the regular accounts.

### Showing assets

We can view what we've deposited inside the Shielder contract (however, note that this gives you virtually no information about who actually put what there). In order to do that, we will need to use the `shielder-cli`. The easiest way to obtain it is to get it from the `cli/target/release/` folder, like so:
```bash
cd cli
./target/release/shielder-cli show-assets
```

The output is not exactly fascinating right now, because we haven't deposited any assets yet. It will contain a line like:
```
assets=[]
```

In order to make it more interesting, we need to deposit some assets: let's!

### Depositing assets

Depositing some PSP22 tokens will be only slightly more involved. We need to run the following command:
```bash
./target/release/shielder-cli deposit 0 10
```

This will deposit 10 units of the first token into Shielder (we have two PSP22 tokens registered with id-s 0 and 1).
It will ask you for the seed of the account depositing the tokens. Type `//0` for Damian and `//1` for Hans (and press 'Enter').

Now, running `show-assets` should be slightly more interesting and show you something like this:
```
assets=[Asset { token_id: 0, token_amount: 10, deposit_id: 0 }]
```
Which seems to make sense, given that we've just deposited 10 _tokens_ into the Shielder.

To make it more interesting, let's deposit more tokens, first as Damian (typing `//0` as the seed):
```bash
./target/release/shielder-cli deposit 0 25
```
Which yields:
```
assets=[Asset { token_id: 0, token_amount: 35, deposit_id: 0 }]
```
(note how the notes have been merged into one).

And later as Hans (`//1`):
```bash
./target/release/shielder-cli deposit 0 33
```

Which in turn yields:
```
assets=[Asset { token_id: 0, token_amount: 68, deposit_id: 0 }]
```

An interesting takeaway from those trivial experiments is that even by inspecting what's inside the Shielder, we can't really say anything useful about the state of the ownership.
Of course, you can try depositing the second token (1) and see a second entry created in the assets lists.

A natural course of action after checking out the deposists would be to check out the withdrawals. Let's do just that!

### Interacting with PSP22 contracts

Before we proceed with our withdrawals, let's figure out a way to inspect the balances of our PSP22 tokens. There are two ways of doing that: the Contracts UI and the `cargo contracts` CLI. We will use the former on account of the simplicity and user-friendliness of the process.
First, navigate to https://contracts-ui.substrate.io/ and in the top-left corner choose 'Local node'.
![Screenshot from 2023-03-29 00-50-28](https://user-images.githubusercontent.com/3109645/228384333-ae302d6e-bd58-47e1-b25c-dcc93c658751.png)

We will need to add our contracts to the UI to be able to interact with them. We will focus on the first contract: adding the second one follows the exact same pattern.
First, let's choose 'Add New Contract' in the top-left corner. Then, when shown a new screen, we will need to choose 'Use On-Chain Contract Address'.

You can learn the contract addresses from the `deploy/addresses.json` file. Let's use `"token_a_address"`, or: `5Ct2Gc8hQscGdhxtso5DP3EWRgk6nww733CpKCiA3qdZ2T8u` (make sure to inspect the file for yourself).
You will need to paste the address in the 'Contract Address' field. Once you do that, two more fields will show up. You can choose any name of your liking for 'Contract Name'. For the metadata, however, you will need to upload the `public_token/target/ink/public_token.json` file.
After completing all that and clicking 'Add contract', you will be able to interact with the PSP22 contract.

To call the contract as either Damian or Hans, you will need to click the `Caller` field and, despite its lack of responsiveness, paste the appropriate address there. Let's use `5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH` (Damian) for the sake of this tutorial.

This will allow you to easily check your balance by selecting the `PSP22::balanceOf` method and again pasting Damian's address. This way we can check the balance before and after the withdrawal from Shielder and be extra sure the the funds actually ended up in the right account.

### Withdrawing assets

Withdrawing assets is in every sense complementary to depositing them. We will run a command and see the funds appear back in our balances of the corresponding PSP22 tokens.

Without further ado, let's run the following command (as Damian, so typing `//0` as the seed):
```bash
/target/release/shielder-cli withdraw --deposit-id 0 --amount 11
```

A natural first question would be: 'how do I get a depost id'? When your run `shielder-cli show-assets` you also see the deposit ids along the tokens you have deposited. In our case, the deposit id will be 0.
You can now inspect the balance of Damian's account using the Contracts UI to verify that the funds indeed made it to his account.

But here comes a major twist and arguably a large chunk of Shielder's power! We can supply an additional argument: `--recipient`. Please read the next section to find out about what value this adds to the whole setup.

### Transferring tokens

As mentioned previously, when withdrawing assets from the Shielder, we can specify the additional `--recipient` argument. What it does is it withdraws the funds not into the caller's account but into the account specified by the argument, thereby supplying us with an extremely useful version of transfers.
Now we can easily transfer some tokens to Hans! All that's needed is running the following command:
```bash
./target/release/shielder-cli withdraw --deposit-id 0 --recipient 5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o --amount 15
```
Note that you will still need to type in the seed of the withdrawer, not the recipient of the transfer (the CLI is quite vocal about this fact, so no need to worry).

You can now check if Hans' balance has been increased accordingly (we bet you 10 TZERO it has!).

### Merging assets
Merging assets allows you to combine two `deposit-id`s into one, preserving the value of the tokens.

Starting from a fresh deployment, you can deposit funds for Damian, following the previously-outlined procedure:

```bash
./target/release/shielder-cli deposit 0 10 
```

While depositing a second batch of tokens for Damian, let us require a new deposit to be created:

```bash
./target/release/shielder-cli deposit 0 15 --require-new-deposit
```

The assets will reflect the split between two `deposit-id`s:

```
assets=[Asset { token_id: 0, token_amount: 10, deposit_id: 0 }, Asset { token_id: 0, token_amount: 15, deposit_id: 1 }]
```

These two deposits can now be merged:
```bash
./target/release/shielder-cli merge 0 1
```

This results in the following state: 
```
assets=[Asset { token_id: 0, token_amount: 25, deposit_id: 0 }]
```

Mind that the merged deposit carries over the `deposit-id` of the first deposit provided to the merge command.

### Closing remarks

If you made it this far: congrats! You've just completed your first foray into the land of privacy and zero-knowledge. Now you're ready to shield your tokens and make transfers through Shielder!
Please make sure to keep an eye on this repository for changes and new features. Also, if you have any questions, don't hesitate to bring them up on the [Aleph Zero Discord](https://discord.com/invite/alephzero).
