# cpm
cpm is a management of competitive programming problem.

# Install

```bash
go get github.com/togatoga/cpm
```

# Examples
### Create Directory based Problem Information
`cpm get` creates directory under `root` and downloads sample cases.
```bash
## Get problem
cpm get http://codeforces.com/contest/417/problem/A

## Get all problems of the contest
cpm get http://codeforces.com/contest/908
```

### Run Test
`cpm run` run test for all test cases.
```bash
# Run test
cpm run ./a.out
```
The `input file`'s name must be `FILENAME_in.txt`.(the sufix must be `_in.txt`)

The `output file`'s name must be `FILENAME_out.txt`.(the sufix must be `_out.txt`)

When you create your own test case. it requires that the `input file` and the `output file` have same `FILENAME`. (ex the `input file` must be custom_in.txt, the `output file` must be custom_out.txt)


### List Created Directories
`cpm list` shows directories under `root`.
```bash
cpm list
```

# Config
Config file is `~/.config/cpm/config.json`
```json
{
   "root": "~/.cpm"
}
```

# Advanced
If you combine [cpm](https://github.com/togatoga/cpm) and interactive search([peco](https://github.com/peco/peco),[fzf](https://github.com/junegunn/fzf)), it's very helpful.
```bash
cd $(cpm list | peco)
```

# Support Sites
- [Codeforces](http://codeforces.com)
- [AtCoder(beta only)](https://beta.atcoder.jp)
