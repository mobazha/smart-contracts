1. Enable and disable the js scripts in ./migrations for migrations
2. Run migration
npm run migrate:polygon

3. Update the new version in remix in Deploy & run transactions view
open https://remix.ethereum.org/
In File view, Select ABI
In Deploy & run transactions view
1) Select Injected Provider - MetaMask
2) Ensure correct account selected
3) In Contract secion, set address "0xb46a91f9546b6650453F2B54705E0e8e25C85247" and click "At Address"
4) getVersionCount("escrow") to have a check
5) Add Version, for example "escrow,v0.9.5,1,0xb26CC1658A44AFDacF806C2Bf8cac07B92764140"
6) markRecommendVersion("escrow,v0.9.5")

4. Send MBZ token from owner account to escrow Contract address for escrow incentive use