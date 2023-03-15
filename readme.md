# Tuffous

Tuffous is a powerful CLI and GUI todo manager.

## Features

- Details, deadline, time, weight and tags support for todos.
- Infinite layers of father/children todos.
- Things-3-like GUI fit with the features of Tuffous.

## Requirements

Tuffous requires a nerd-font patched font in order to display icons in CLI normally.

In GUI you don't need a special font.

## Usage

To use Tuffous, you need first initialize a new todo repo using `init` in order to store todos.

Tuffous is path-based, so you need to run it in the target folder you want.

### Commands

```
init        Initialize a new todo repo
new         Create a new todo
list        List todo(s) with filter(s)
edit        Edit todo(s) with filter(s)
complete    Complete todo(s) with filter(s)
father      Mark a todo as father with filter(s) in the cache
child       Mark todo(s) as children with filter(s) in the cache
remove      Remove todo(s) as children with filter(s)
cleancache  Clean cache
gui         Open GUI (WIP)
help        Print this message or the help of the given subcommand(s)
```

### General Command arguments

Filter arguments:

```
--ftoday <TODAY>                    Filter with today only todo(s) [default: false]
--fdate <DATE>                      Filter with date-only todo(s)
--fdater <DATE_RANGE> <DATE_RANGE>  Filter with ranged date-only todo(s)
--fddl <DDL>                        Filter with ddl-only todo(s)
--fddlr <DDL_RANGE> <DDL_RANGE>     Filter with ranged ddl-only todo(s)
--flogged <LOGGED>                  Filter with logged todo(s) [default: false]
--ftag <TAGS>                       Filter with tags
--fname <NAME>                      Search with name
```

Edit arguments:

```
-n, --name <NAME>                       Change name of the target
-d, --details <DETAILS>                 Change details of the target
-w, --date <DATE>                       Change date of the target
    --ddl <DEADLINE>                    Change deadline of the target
    --weight <WEIGHT>                   Change weight of the target
-t, --tag <TAGS>                        Bind/unbind tags for the target
-c, --complete <BOOLEAN>                Complete/uncomplete the target
```

Help argument:

```
-h, --help  Print help
```
