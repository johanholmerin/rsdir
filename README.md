# rsdir

Edit directories as a text file. Rust reimplementation of vidir from
[moreutils](https://joeyh.name/code/moreutils/), with some features from the
[fork by trapd00r](https://github.com/trapd00r/vidir)

## Installation

```sh
cargo install rsdir
```

## Usage

```sh
# Say you have a directory that looks like this
├─ old/
│  ├─ file1
├─ file1
└─ file_2

# Running rsdir will open your editor with the following content
1 ./file1
2 ./file_2
3 ./old/

# The numbers are used to keep track of each file - editing a path will
# rename the file/directory while removing a line will delete it

# We can the remove the _ from the second file to make the naming consistent
# and remove the third line, leaving us with the following
1 ./file1
2 ./file2

# Save the file and exit the editor and rsdir will perform the modifications
├─ file1
└─ file2

# If you ran with the --verbose flag you would get the following log
Moved file "./file_2" to "./file2"
Removed directory "./old"
```

## Examples

```sh
# Defaults to currenty directory
rsdir

# Supports multiple directories
rsdir ./foo ../bar

# Verbose mode will log what files are moved/deleted
rsdir --verbose

# Use another editor. Will default to vi if EDITOR isn't set
EDITOR=nano rsdir
```
