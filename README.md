```
GitHub Branch Table 1.0
Zero <https://github.com/ZeroErrors>
Generates a table output of branches for multiple repositories allowing for easy comparison.

USAGE:
    github_branch_table [FLAGS] [OPTIONS] <REPOS>...

FLAGS:
    -c, --cache      Enable the repo cache
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --token <token>    GitHub token used when making API requests

ARGS:
    <REPOS>...    GitHub repo's to include in the table
```


Example:
```
github_branch_table ZeroErrors/github_branch_table ZeroErrors/Samaritan
```

```
Get branches in ZeroErrors/github_branch_table (page 1)
Rate Limit: 5000, Remaining: 4999, Reset: 1548280887
Get branches in ZeroErrors/Samaritan (page 1)
Rate Limit: 5000, Remaining: 4996, Reset: 1548280887
+--------+----------------------+------------+-----------+----------------------+------------+--------+
| Branch | ZeroErrors/github_branch_table                | ZeroErrors/Samaritan                       |
+--------+----------------------+------------+-----------+----------------------+------------+--------+
|        | Last Updated         | Updated By | PR        | Last Updated         | Updated By | PR     |
+--------+----------------------+------------+-----------+----------------------+------------+--------+
| master | 2019-01-23T20:41:39Z | ZeroErrors |           | 2018-02-14T02:54:18Z | ZeroErrors |        |
+--------+----------------------+------------+-----------+----------------------+------------+--------+
```
