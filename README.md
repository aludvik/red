# Red

Red is a text editor, specialized for editing source code.

## Goals

1. Zero Configuration

   Red works the right way out-of-the-box.

2. Most Important Features

   Red has the right features for writing software.

3. Small

   Red has a minimal install size and uses minimal resources.

4. Simple

   Red is simple to install and start using.

## Editing

Red is a modal text editor. The editor starts in "normal" mode.

To open a new or existing file for editing, just type `red [file]`.

The clipboard used for cutting and pasting lines is a stack.

### Normal Mode

- `j`, `k`, `l`, `h`: Move the cursor
- `J`, `K`, `L`, `H`: Move the cursor between whitespace
- `i`: Enter insert mode
- `d`: Delete the current line
- `x`: Cut the current line and insert it into the clipboard
- `c`: Copy the current line and insert it into the clipboard
- `v`: Insert the top line from the clipboard and remove it from the clipboard
- `s`: Save the file
- `q`: Quit

### Insert Mode

- `Escape`: Enter normal mode

## FAQ

1. Why write a new text editor?

   I love vim, but sometimes it is a bit much. Red is a love letter to vim
   asking it why everything had to become so complicated.

2. Why red?

   - (R)ed (Ed)itor
   - [(RED)](https://www.red.org/donate)
   - (R)adical (Ed)ward
   - (R)ust Text (Ed)itor
   - (Red) Forman
