## GhostChain Refactor from zig 

## TODO 
- [x] Remove zig code and revert to rust base 
- [ ] Review archive-zig/ folder and all zig projects in there
- [ ] We'll be leveraging mostly rust besides Ghostplane our L2 Which will be built in zig 
- [ ] Zqlite our crypto  database we'll use thats at github.com/ghostkellz/zqlite
- [ ] We'll need to rebuild those zig projects in pure rust like zns, zvm, etc. And come up with new names? ZNS GNS for Ghost Name Server? idk we'll need to plan the names
- [ ] We'll have to look at what's WORTH keeping in zig and what should  be pure rust
- [ ] We'll keep a monolithhic workspace so folder for GNS say and RVM (rust virtual machine) etc. 
- [ ] Plan the best path forward to get our project operational and working agaIN
- [ ] GCRYPT is our crypto backbone - Github.com/ghostkellz/gcrypt  written in rust can be added as a crate
- [ ] GQUIC - our quic/http3 etc. operations at github.com/ghostkellz/gquic
- [ ] Review all project .md FILES to see our original plan and direction
- [ ] We're no longer building ghostchain in zig, Rust variant only with GhostPlane Layer 2 in zig
- [ ] Not sure RNS or GNS as ZNS replacement, RNS for ZVM and zsig zwallet and zledger replaced into this repo in rust as sig, wallet,walletd, ledger, ledgerd, Keep Keystone functionality as keyston in our monolithick repo here
- [ ] Want a modular workspace 
- [ ] dont want to lose the Ghostchain vision in all this 

- [ ] ZNS is now CNS - Crypto Name server - Rust based
- [ ] ZVM is now RVM - Rust Virtual Machine - github.com/ghostkellz/rvm
- [ ] ZSIG is now RSIG - Rust Signer - will be a part of this workspace
- [ ] ZWALLET is now GWALLET - Ghost Wallet - Will be part of this workspace 
- [ ] ZLEDGER is now GLEDGER - GhostChain Ledger - will be part of this workspace
- [ ] Let's create a archived-docs section and move the zig based docs there and recreate our rust variants here in this repo
- [ ] Ghostplane is still going to be a L2 Blockchain in zig so we'll still need a Zig bridge from rust I take it
- [ ] All the concepts remain the same just not full zig base - just Ghostplane L2 in zig and maybe ghostbridge in rust or zig depending on what you think
- [ ] zquic and zcrypto will use the gquic and gcrypt crates in rust 
- [ ] I added new logos for ghosstchain - gcc-primary-logo, gcc-credits-logo, gcc-secondary-logo etc. in the asset folders. 
- [ ] Tokens now have a icon too 
- [ ] We're still leveraging Jarvis for AI stuff 
- [ ] Shroud and ZID functionality will be GID - Ghost ID system which will be part of this workspace 
- [ ] Keystone will remain as Keystone but Now it's rust based, We can keep that in our Ghostchain repo probably safer? 
- [ ] Update Readme file to have gcc-logo-primary.png  At the top 
- [ ] Let's really clean up. ZQLITE that crypto based database still is a zig project so we can keep using that it's rust capable via FFI binding etc. github.com/ghostkellz/zqlite 
- [ ] I added archive directory for old documentation that's been replaced etc. archive-zig contains the old zig projects

