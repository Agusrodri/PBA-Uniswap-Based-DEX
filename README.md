# Week 3 Assignment - FRAME

This assignment is a project covering the material you've learned about writing Substrate Runtimes with FRAME. To complete this project, select one of the following options, and implement the runtime described using FRAME.

## Option 1: Decentralized Exchange

- Use a multi-asset pallet to represent many tokens.
  - You can create your own, or use the assets pallet provided in template.
- Create a Uniswap style DEX to allow users to trustlessly exchange tokens with one another.
  - Implement rewards to incentivize users to create pools.
  - Expose an API from the pallet which acts as a “price oracle” based on the existing liquidity pools.

Extras:

- Create some kind of asset or marketplace where users can use any token to purchase resources, using the price oracle to make sure users pay enough.
- Integrate other DeFi utilities on top of your DEX.

## Option 2: Quadratic Voting

- Create a simple Identity system to "de-sybil" your users.
  - Could be integrated into your pallet, a secondary pallet, or use the existing identity pallet provided by FRAME.
- Create a voting system where users with identities can reserve / lock an amount of token, which then weights their vote on a quadratic scale.
- Proposals should be a simple on chain text or hash, votes can be simply aye or nay.

Extras:

- Create a more complex proposal system where users can vote on multiple things at once, and have to consider how they want to distribute their votes across them.
- Allow proposals to dispatch on-chain calls and make changes to your runtime.

## Option 3: Direct Delegation Proof of Stake

- The pallet should have logic to manage "validators" and "delegators".
  - Validators register themselves as potential block producers.
  - Any other user can use their tokens to delegate (vote) for the set of validators they want.
- Where every N blocks (a session), the current “winners” are selected, and Aura is updated.
- Block rewards should be given to the current block producer and the delegators who backed them.

Extras:

- Try to support delegation chains, where a delegator can delegate to another delegator.
- Think about and implement some kind of slashing for validators if they “misbehave”.
- Integrate the Session pallet rather than using Aura directly.

## Grading rubric

Your implementation will be reviewed for code quality and implementation details.

- Implementation
  - Correctness and accuracy of implementation
  - Evidence of using various techniques used in class
  - As close to production ready as possible
- Code Quality
  - Tests and code coverage
  - Use of best practices and efficient code
  - Well documented, with considerations and compromises noted
- Bonus Points
  - Integrate this into a working node.
  - UX to interact with the runtime code.
    - Value functionality over beauty.
  - Add something benchmarking and weights.

### Review Process

Here is a little insight into how we will grade and review your project.

1. Reading your project description.

   Your README file should describe:

   - The project you selected.
   - Important details, background, and considerations you researched.
   - How you designed your state transition function.
   - Any compromises you made, or things you would improve if you had time.
   - etc...

   Basically, we should be able to understand what to expect to see before we read any of your code.

2. Reading through your code.

   - We will scan through your extrinsics and try to map the code your wrote to the state transition function you described in your README.
   - We will look to make sure you are following best practices and that your code is safe / non-exploitable by malicious actors.
   - Finally we will look at cleanliness and code quality.
   - It goes without saying, but we will also make sure that the code written is your own, and if we ask, you should be able to explain to us how things are working, and why you chose to implement things in the way you did.

3. Running and reading your tests.

   You should have a comprehensive test suite which covers all the successful and "error" flows through your state transition function.

   - All of the mainline paths of your state transition function should be covered and shown to execute correctly, with checks of your final state.
   - Any paths which are not successful for your state transition function should have well handled errors, and tests should show that is the case.

4. Looking for any work / ideas above and beyond the basic project description.

   You should prioritize getting your project working above all else. However, if you find yourself with extra time and ambition, you will find that all of the project choices have a lot of room for you to design beyond what we have asked for.

   Use your knowledge of cryptography, game theory, blockchain, and existing useful applications to extend and improve your project.

   Make your project as close to "production ready" as possible, for example considering deeply the economics of your system, punishments for bad actors, benchmarking, etc....

   Consider breaking your project into multiple modular parts which work together and are more reusable.

   If you do not have time to actually program all of your ambitions, spend the time to describe to us what you would like to do, and how you would have done it.
