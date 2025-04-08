# Rust Macro Analyzer
This is the tool that was developed for my graduation thesis: **Macro Usage in the Rust Programming Language in Open Source Repositories**\

You can read the thesis here: TODO

## Abstract
This work presents an analysis of macro usage in the 100 most popular open-source Rust projects hosted on GitHub. The study aims to uncover patterns in the use of macros, which play a critical role in the Rust ecosystem by providing metaprogramming capabilities and improving code efficiency. By scraping project repositories, we identify the types of macros most frequently employed, analyze which projects rely heavily on macros, and delve into specific categories to determine the most commonly used macros. The results offer valuable insights into macro adoption trends in the Rust community and may assist developers in understanding how and why macros are applied in large-scale projects.

## How to run

First, you need a Github token with the `repo` scope to access the GitHub API. You can create this token by following the instructions [here](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token).

Then, to start the analysis, run the following command:
```bash
GITHUB_TOKEN=<your-github-token> cargo run
```
Note: The first time you run the tool, it will take some time to download the data from GitHub. Subsequent runs will be much faster.

Finally, to see the results, open another terminal on the `web` folder and run the following commands:

```bash
yarn i
yarn start
```
Note: You can use `npm` instead of `yarn` if you prefer.
